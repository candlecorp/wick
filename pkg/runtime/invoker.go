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
	"encoding/binary"
	"encoding/json"
	"fmt"
	"reflect"

	"github.com/go-logr/logr"
	"github.com/nanobus/iota/go/operations"
	"github.com/nanobus/iota/go/payload"
	"github.com/nanobus/iota/go/rx/flux"
	"github.com/nanobus/iota/go/rx/mono"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/channel"
	"github.com/nanobus/nanobus/pkg/compute"
	"github.com/nanobus/nanobus/pkg/logger"
	"github.com/nanobus/nanobus/pkg/security/claims"
	"github.com/nanobus/nanobus/pkg/stream"
)

type Invoker struct {
	log logr.Logger
	compute.Invoker
	codec     channel.Codec
	ns        Namespaces
	ops       operations.Table
	runnables []Runnable
	targets   []Target
}

type Target struct {
	Namespace string
	Operation string
}

func (i Target) String() string {
	return fmt.Sprintf("%s::%s", i.Namespace, i.Operation)
}

func NewInvoker(log logr.Logger, ns Namespaces, codec channel.Codec) *Invoker {
	ops := make(operations.Table, 0, 10)
	runnables := make([]Runnable, 0, 10)
	targets := make([]Target, 0, 10)

	index := uint32(0)
	for namespace, functions := range ns {
		for operation, r := range functions {
			targets = append(targets, Target{
				Namespace: namespace,
				Operation: operation,
			})
			ops = append(ops, operations.Operation{
				Index:     index,
				Type:      operations.RequestResponse,
				Direction: operations.Export,
				Namespace: namespace,
				Operation: operation,
			})
			ops = append(ops, operations.Operation{
				Index:     index,
				Type:      operations.FireAndForget,
				Direction: operations.Export,
				Namespace: namespace,
				Operation: operation,
			})
			ops = append(ops, operations.Operation{
				Index:     index,
				Type:      operations.RequestStream,
				Direction: operations.Export,
				Namespace: namespace,
				Operation: operation,
			})
			ops = append(ops, operations.Operation{
				Index:     index,
				Type:      operations.RequestChannel,
				Direction: operations.Export,
				Namespace: namespace,
				Operation: operation,
			})
			runnables = append(runnables, r)
			index++
		}
	}

	return &Invoker{
		log:       log,
		codec:     codec,
		ns:        ns,
		ops:       ops,
		runnables: runnables,
		targets:   targets,
	}
}

func (i *Invoker) Close() error { return nil }

func (i *Invoker) Operations() operations.Table {
	return i.ops
}

func (i *Invoker) FireAndForget(ctx context.Context, p payload.Payload) {
	r, data := i.lookup(ctx, p)
	go func() {
		if _, err := r(ctx, data); err != nil {
			logger.Error("error with FireAndForget request", "error", err)
		}
	}()
}

func (i *Invoker) RequestResponse(ctx context.Context, p payload.Payload) mono.Mono[payload.Payload] {
	r, data := i.lookup(ctx, p)
	return mono.Create(func(sink mono.Sink[payload.Payload]) {
		go func() {
			result, err := r(ctx, data)
			if err != nil {
				sink.Error(err)
				return
			}

			if isNil(result) {
				sink.Success(payload.New(nil))
				return
			}

			data, err := i.codec.Encode(result)
			if err != nil {
				sink.Error(err)
				return
			}

			sink.Success(payload.New(data))
		}()
	})
}

func (i *Invoker) RequestStream(ctx context.Context, p payload.Payload) flux.Flux[payload.Payload] {
	r, data := i.lookup(ctx, p)
	return flux.Create(func(sink flux.Sink[payload.Payload]) {
		go func() {
			s := stream.FromSink(sink)
			ctx = stream.SinkNewContext(ctx, s)
			_, err := r(ctx, data)
			if err != nil {
				sink.Error(err)
				return
			}

			// if isNil(result) {
			// 	sink.Next(payload.New(nil))
			// 	return
			// }

			sink.Complete()
		}()
	})
}

func (i *Invoker) RequestChannel(ctx context.Context, p payload.Payload, in flux.Flux[payload.Payload]) flux.Flux[payload.Payload] {
	r, data := i.lookup(ctx, p)
	return flux.Create(func(sink flux.Sink[payload.Payload]) {
		go func() {
			streamSink := stream.FromSink(sink)
			ctx = stream.SinkNewContext(ctx, streamSink)
			streamSource := stream.SourceFromFlux(i.codec, in)
			ctx = stream.SourceNewContext(ctx, streamSource)

			_, err := r(ctx, data)
			if err != nil {
				sink.Error(err)
				return
			}

			// if isNil(result) {
			// 	sink.Next(payload.New(nil))
			// }

			sink.Complete()
		}()
	})
}

func (i *Invoker) lookup(ctx context.Context, p payload.Payload) (Runnable, actions.Data) {
	md := p.Metadata()
	index := binary.BigEndian.Uint32(md)
	r := i.runnables[index]
	t := i.targets[index]
	var input interface{}
	if err := i.codec.Decode(p.Data(), &input); err != nil {
		// TODO(jsoverson): improve error logging. This bypasses a log error if we got
		// no payload data but we should log an error if we expected some.
		if len(p.Data()) > 0 {
			logger.Warn("received error when decoding payload", "action", t.String(), "error", err)
		}
	}
	c := claims.FromContext(ctx)
	data := actions.Data{
		"input":  input,
		"$":      input,
		"pipe":   input,
		"claims": c,
	}

	if jsonBytes, err := json.MarshalIndent(input, "", "  "); err == nil {
		logOutbound(t.String(), string(jsonBytes))
	}

	return r, data
}

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
}

func logOutbound(target string, data string) {
	logger.Debug("<== " + target + " " + data)
}
