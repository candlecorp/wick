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
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resiliency/breaker"
	"github.com/nanobus/nanobus/pkg/resiliency/retry"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type Environment map[string]string

type Processor struct {
	ctx             context.Context
	log             logr.Logger
	tracer          trace.Tracer
	config          *Configuration
	registry        actions.Registry
	resolver        resolve.DependencyResolver
	resolveAs       resolve.ResolveAs
	timeouts        map[string]time.Duration
	retries         map[string]*retry.Config
	circuitBreakers map[string]*breaker.CircuitBreaker
	services        Namespaces
	providers       Namespaces
	events          Functions
	pipelines       Functions
}

type Namespaces map[string]Functions
type Functions map[string]Runnable

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

func NewProcessor(ctx context.Context, log logr.Logger, tracer trace.Tracer, configuration *Configuration, registry actions.Registry, resolver resolve.DependencyResolver) (*Processor, error) {
	timeouts := make(map[string]time.Duration, len(configuration.Resiliency.Timeouts))
	for name, d := range configuration.Resiliency.Timeouts {
		timeouts[name] = time.Duration(d)
	}

	retries := make(map[string]*retry.Config, len(configuration.Resiliency.Retries))
	for name, retryMap := range configuration.Resiliency.Retries {
		retryConfig, err := retry.DecodeConfig(retryMap)
		if err != nil {
			return nil, err
		}
		retries[name] = &retryConfig
	}

	circuitBreakers := make(map[string]*breaker.CircuitBreaker, len(configuration.Resiliency.CircuitBreakers))
	for name, circuitBreaker := range configuration.Resiliency.CircuitBreakers {
		cb := breaker.CircuitBreaker{
			Name: name,
		}
		if err := config.Decode(circuitBreaker, &cb); err != nil {
			return nil, err
		}
		cb.Initialize(log)
		circuitBreakers[name] = &cb
	}

	p := Processor{
		ctx:             ctx,
		log:             log,
		tracer:          tracer,
		config:          configuration,
		timeouts:        timeouts,
		retries:         retries,
		circuitBreakers: circuitBreakers,
		registry:        registry,
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

func (p *Processor) Service(ctx context.Context, namespace, service, function string, data actions.Data) (interface{}, bool, error) {
	s, ok := p.services[namespace+"."+service]
	if !ok {
		return nil, false, nil
	}

	pl, ok := s[function]
	if !ok {
		return nil, false, nil
	}

	output, err := pl(ctx, data)
	return output, true, err
}

func (p *Processor) GetProviders() Namespaces {
	return p.providers
}

func (p *Processor) Provider(ctx context.Context, namespace, service, function string, data actions.Data) (interface{}, error) {
	nss := namespace + "." + service
	s, ok := p.providers[nss]
	if !ok {
		return nil, fmt.Errorf("provider %q not found", nss)
	}

	pl, ok := s[function]
	if !ok {
		return nil, fmt.Errorf("function %q in provider %q not found", function, nss)
	}

	return pl(ctx, data)
}

func (p *Processor) Event(ctx context.Context, name string, data actions.Data) (interface{}, error) {
	pl, ok := p.events[name]
	if !ok {
		return nil, fmt.Errorf("unknown event name %q", name)
	}

	return pl(ctx, data)
}

func (p *Processor) Pipeline(ctx context.Context, name string, data actions.Data) (interface{}, error) {
	pl, ok := p.pipelines[name]
	if !ok {
		return nil, fmt.Errorf("unknown pipeline name %q", name)
	}

	return pl(ctx, data)
}

func (p *Processor) Initialize() (err error) {
	if p.pipelines, err = p.loadFunctionPipelines(p.config.Pipelines); err != nil {
		return err
	}
	if p.services, err = p.loadServices(p.config.Services); err != nil {
		return err
	}
	if p.providers, err = p.loadServices(p.config.Providers); err != nil {
		return err
	}
	if p.events, err = p.loadFunctionPipelines(p.config.Events); err != nil {
		return err
	}

	return nil
}

func (p *Processor) loadServices(services Services) (s Namespaces, err error) {
	s = make(Namespaces, len(services))
	for ns, fns := range services {
		if s[ns], err = p.loadFunctionPipelines(fns); err != nil {
			return nil, err
		}
	}
	return s, nil
}

func (p *Processor) loadFunctionPipelines(fpl FunctionPipelines) (Functions, error) {
	runnables := make(Functions, len(fpl))
	for name, pipeline := range fpl {
		pl, err := p.LoadPipeline(&pipeline)
		if err != nil {
			return nil, err
		}
		runnables[name] = pl
	}

	return runnables, nil
}

func (p *Processor) LoadPipeline(pl *Pipeline) (Runnable, error) {
	if pl.Call != "" {
		return func(ctx context.Context, data actions.Data) (output interface{}, err error) {
			return p.Pipeline(ctx, pl.Call, data)
		}, nil
	}

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
	if s.Call != "" {
		action = func(ctx context.Context, data actions.Data) (output interface{}, err error) {
			return p.Pipeline(ctx, s.Call, data)
		}
	} else {
		loader, ok := p.registry[s.Uses]
		if !ok {
			return nil, fmt.Errorf("unregistered action %q", s.Uses)
		}

		action, err = loader(p.ctx, s.With, p.resolveAs)
		if err != nil {
			return nil, err
		}
	}

	var retry *retry.Config
	if s.Retry != "" {
		var ok bool
		retry, ok = p.retries[s.Retry]
		if !ok {
			return nil, fmt.Errorf("retry policy %q is not defined", s.Retry)
		}
	}

	var circuitBreaker *breaker.CircuitBreaker
	if s.CircuitBreaker != "" {
		var ok bool
		circuitBreaker, ok = p.circuitBreakers[s.CircuitBreaker]
		if !ok {
			return nil, fmt.Errorf("circuit breaker policy %q is not defined", s.CircuitBreaker)
		}
	}

	var timeout time.Duration
	if s.Timeout != "" {
		if named, exists := p.timeouts[s.Timeout]; exists {
			timeout = named
		} else {
			to, err := time.ParseDuration(s.Timeout)
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

		if s.config.Returns != "" {
			data[s.config.Returns] = output
		}

		if output != nil {
			runOutput = output
		}
	}

	return runOutput, nil
}
