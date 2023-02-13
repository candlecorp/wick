/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package runtime

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/cenkalti/backoff/v4"
	"github.com/go-logr/logr"
	"go.opentelemetry.io/otel/trace"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resiliency/breaker"
	"github.com/nanobus/nanobus/pkg/resiliency/retry"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type Environment map[string]string

type Processor struct {
	ctx        context.Context
	log        logr.Logger
	tracer     trace.Tracer
	registry   actions.Registry
	resolver   resolve.DependencyResolver
	resolveAs  resolve.ResolveAs
	resiliency *resiliency.Policies
	interfaces Namespaces
	providers  Namespaces
}

type Namespaces map[string]Functions
type Functions map[string]Runnable

func (ns Namespaces) Invoke(ctx context.Context, h handler.Handler, data actions.Data) (interface{}, bool, error) {
	s, ok := ns[h.Interface]
	if !ok {
		return nil, false, nil
	}

	pl, ok := s[h.Operation]
	if !ok {
		return nil, false, nil
	}

	output, err := pl(ctx, data)
	return output, true, err
}

type Runnable func(ctx context.Context, data actions.Data) (interface{}, error)

type runnable struct {
	log    logr.Logger
	tracer trace.Tracer
	config *Pipeline
	steps  []step
}

type step struct {
	config         *Step
	action         actions.Action
	timeout        time.Duration
	retry          *retry.Config
	circuitBreaker *breaker.CircuitBreaker
	onError        Runnable
}

var noopResiliency = resiliency.Policies{
	Timeouts:        make(map[string]time.Duration, 0),
	Retries:         make(map[string]*retry.Config, 0),
	CircuitBreakers: make(map[string]*breaker.CircuitBreaker, 0),
}

func NewProcessor(ctx context.Context, log logr.Logger, tracer trace.Tracer, registry actions.Registry, resolver resolve.DependencyResolver) (*Processor, error) {
	p := Processor{
		ctx:        ctx,
		log:        log,
		tracer:     tracer,
		registry:   registry,
		resiliency: &noopResiliency,
		interfaces: make(Namespaces),
		providers:  make(Namespaces),
	}

	p.resolver = func(name string) (interface{}, bool) {
		if name == "system:processor" {
			return &p, true
		}
		return resolver(name)
	}

	p.resolveAs = resolve.ToResolveAs(p.resolver)

	return &p, nil
}

func (p *Processor) GetInterfaces() Namespaces {
	return p.interfaces
}

func (p *Processor) Interface(ctx context.Context, h handler.Handler, data actions.Data) (interface{}, bool, error) {
	s, ok := p.interfaces[h.Interface]
	if !ok {
		return nil, false, nil
	}

	pl, ok := s[h.Operation]
	if !ok {
		return nil, false, nil
	}

	output, err := pl(ctx, data)
	return output, true, err
}

func (p *Processor) GetProviders() Namespaces {
	return p.providers
}

func (p *Processor) Provider(ctx context.Context, h handler.Handler, data actions.Data) (interface{}, bool, error) {
	s, ok := p.providers[h.Interface]
	if !ok {
		return nil, false, nil
	}

	pl, ok := s[h.Operation]
	if !ok {
		return nil, false, nil
	}

	output, err := pl(ctx, data)
	return output, true, err
}

func (r *Resiliency) ToPolicies(log logr.Logger) (*resiliency.Policies, error) {
	p := resiliency.Policies{
		Timeouts:        make(map[string]time.Duration),
		Retries:         make(map[string]*retry.Config),
		CircuitBreakers: make(map[string]*breaker.CircuitBreaker),
	}

	if r == nil {
		return &p, nil
	}

	for name, d := range r.Timeouts {
		p.Timeouts[name] = time.Duration(d)
	}

	for name, retryMap := range r.Retries {
		retryConfig, err := ConvertBackoffConfig(retryMap)
		if err != nil {
			return nil, err
		}
		p.Retries[name] = &retryConfig
	}

	for name, circuitBreaker := range r.CircuitBreakers {
		cb := breaker.CircuitBreaker{
			Name: name,
		}
		if err := config.Decode(circuitBreaker, &cb); err != nil {
			return nil, err
		}
		cb.Initialize(log)
		p.CircuitBreakers[name] = &cb
	}

	return &p, nil
}

func (p *Processor) Initialize(configuration *BusConfig) (err error) {
	p.resiliency, err = configuration.Resiliency.ToPolicies(p.log)
	if err != nil {
		return err
	}

	providers, err := p.loadInterfaces(configuration.Providers)
	if err != nil {
		return err
	}
	for k, v := range providers {
		p.providers[k] = v
	}

	interfaces, err := p.loadInterfaces(configuration.Interfaces)
	if err != nil {
		return err
	}
	for k, v := range interfaces {
		p.interfaces[k] = v
	}

	return nil
}

func (p *Processor) Resiliency() *resiliency.Policies {
	return p.resiliency
}

func (p *Processor) SetResiliency(policies *resiliency.Policies) {
	p.resiliency = policies
}

func (p *Processor) LoadProviders(ifaces Interfaces) error {
	providers, err := p.loadInterfaces(ifaces)
	if err != nil {
		return err
	}
	for k, v := range providers {
		p.providers[k] = v
	}
	return nil
}

func (p *Processor) LoadInterfaces(ifaces Interfaces) error {
	interfaces, err := p.loadInterfaces(ifaces)
	if err != nil {
		return err
	}
	for k, v := range interfaces {
		p.interfaces[k] = v
	}
	return nil
}

func (p *Processor) loadInterfaces(services Interfaces) (s Namespaces, err error) {
	s = make(Namespaces, len(services))
	for ns, fns := range services {
		if s[ns], err = p.loadFunctionPipelines(fns); err != nil {
			return nil, err
		}
	}
	return s, nil
}

func (p *Processor) loadFunctionPipelines(fpl Operations) (Functions, error) {
	runnables := make(Functions, len(fpl))
	for name, pipeline := range fpl {
		pl, err := p.LoadPipeline(&pipeline)
		if err != nil {
			return nil, fmt.Errorf("could not load pipeline %q: %w", name, err)
		}
		runnables[name] = pl
	}

	return runnables, nil
}

func (p *Processor) LoadPipeline(pl *Pipeline) (Runnable, error) {
	steps := make([]step, len(pl.Steps))
	for i := range pl.Steps {
		s := &pl.Steps[i]
		if s.Name == "" {
			s.Name = fmt.Sprintf("Step %d", i)
		}
		step, err := p.loadStep(s)
		if err != nil {
			return nil, err
		}
		steps[i] = *step
	}

	r := runnable{
		log:    p.log,
		tracer: p.tracer,
		config: pl,
		steps:  steps,
	}

	return r.Run, nil
}

func (p *Processor) loadStep(s *Step) (*step, error) {
	var err error
	var action actions.Action

	loader, ok := p.registry[s.Uses]
	if !ok {
		return nil, fmt.Errorf("unregistered action %q", s.Uses)
	}

	action, err = loader(p.ctx, s.With, p.resolveAs)
	if err != nil {
		return nil, err
	}
	// }

	var retry *retry.Config
	if s.Retry != nil {
		var ok bool
		retry, ok = p.resiliency.Retries[*s.Retry]
		if !ok {
			return nil, fmt.Errorf("retry policy %q is not defined", *s.Retry)
		}
	}

	var circuitBreaker *breaker.CircuitBreaker
	if s.CircuitBreaker != nil {
		var ok bool
		circuitBreaker, ok = p.resiliency.CircuitBreakers[*s.CircuitBreaker]
		if !ok {
			return nil, fmt.Errorf("circuit breaker policy %q is not defined", *s.CircuitBreaker)
		}
	}

	var timeout time.Duration
	if s.Timeout != nil {
		if named, exists := p.resiliency.Timeouts[*s.Timeout]; exists {
			timeout = named
		} else {
			to, err := time.ParseDuration(*s.Timeout)
			if err != nil {
				return nil, err
			}
			timeout = to
		}
	}
	var onError Runnable
	if s.OnError != nil {
		onError, err = p.LoadPipeline(s.OnError)
		if err != nil {
			return nil, err
		}
	}

	return &step{
		config:         s,
		action:         action,
		timeout:        timeout,
		retry:          retry,
		circuitBreaker: circuitBreaker,
		onError:        onError,
	}, nil
}

func (r *runnable) Run(ctx context.Context, data actions.Data) (interface{}, error) {
	var runOutput interface{}
	var err error
	for _, s := range r.steps {
		var output interface{}
		rp := resiliency.Policy(r.log, s.config.Name, s.timeout, s.retry, s.circuitBreaker)
		err = rp(ctx, func(ctx context.Context) error {
			var span trace.Span
			ctx, span = r.tracer.Start(ctx, s.config.Name)
			defer span.End()
			output, err = s.action(ctx, data)
			if errors.Is(err, actions.ErrStop) {
				return backoff.Permanent(err)
			}
			return err
		})
		if err != nil {
			var pe *backoff.PermanentError
			if errors.As(err, &pe) {
				err = pe.Err
			}
			if errors.Is(err, actions.ErrStop) {
				return nil, nil
			}
			return nil, err
		}
		if err != nil && s.onError != nil {
			if output, err = s.onError(ctx, data); err != nil {
				if errors.Is(err, actions.ErrStop) {
					return nil, nil
				}
				return nil, err
			}
		}

		if s.config.Returns != nil {
			data[*s.config.Returns] = output
		}

		if output != nil {
			runOutput = output

			// The `$` and `pipe` variables carries on throughout the pipeline.
			data["$"] = output
			data["pipe"] = output
		}
	}

	return runOutput, nil
}
