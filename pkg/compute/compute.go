/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package compute

import (
	"context"
	"io"

	"github.com/nanobus/iota/go/invoke"
	"github.com/nanobus/iota/go/operations"
	"github.com/nanobus/iota/go/payload"
	"github.com/nanobus/iota/go/rx/flux"
	"github.com/nanobus/iota/go/rx/mono"

	"github.com/nanobus/nanobus/pkg/registry"
)

type (
	NamedLoader = registry.NamedLoader[Invoker]
	Loader      = registry.Loader[Invoker]
	Registry    = registry.Registry[Invoker]

	BusInvoker   func(ctx context.Context, namespace, service, function string, input interface{}) (interface{}, error)
	StateInvoker func(ctx context.Context, namespace, id, key string) ([]byte, error)

	Invoker interface {
		io.Closer
		Operations() operations.Table

		FireAndForget(context.Context, payload.Payload)
		RequestResponse(context.Context, payload.Payload) mono.Mono[payload.Payload]
		RequestStream(context.Context, payload.Payload) flux.Flux[payload.Payload]
		RequestChannel(context.Context, payload.Payload, flux.Flux[payload.Payload]) flux.Flux[payload.Payload]

		SetRequestResponseHandler(index uint32, handler invoke.RequestResponseHandler)
		SetFireAndForgetHandler(index uint32, handler invoke.FireAndForgetHandler)
		SetRequestStreamHandler(index uint32, handler invoke.RequestStreamHandler)
		SetRequestChannelHandler(index uint32, handler invoke.RequestChannelHandler)
	}
)
