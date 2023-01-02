/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core

import (
	"context"
	"encoding/json"

	"github.com/nanobus/iota/go/payload"
	"github.com/nanobus/iota/go/rx/mono"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type Invoker interface {
	RequestResponse(ctx context.Context, h handler.Handler, p payload.Payload) mono.Mono[payload.Payload]
}

func InvokeLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := InvokeConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var m Invoker
	if err := resolve.Resolve(resolver,
		"compute:mesh", &m); err != nil {
		return nil, err
	}

	return InvokeAction(m, &c), nil
}

func InvokeAction(
	invoker Invoker,
	config *InvokeConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		input := data["input"]
		if config.Input != nil {
			var err error
			input, err = config.Input.Eval(data)
			if err != nil {
				return nil, err
			}
		}

		switch v := input.(type) {
		case []byte:
			if err := json.Unmarshal(v, &input); err != nil {
				return nil, err
			}
		case string:
			if err := json.Unmarshal([]byte(v), &input); err != nil {
				return nil, err
			}
		}

		var h handler.Handler
		if config.Handler != nil {
			h = *config.Handler
		} else {
			h = handler.FromContext(ctx)
		}

		payloadData, err := msgpack.Marshal(input)
		if err != nil {
			return nil, err
		}

		metadata := make([]byte, 8)
		p := payload.New(payloadData, metadata)

		result, err := invoker.RequestResponse(ctx, h, p).Block()
		if err != nil {
			return nil, err
		}

		if len(result.Data()) > 0 {
			var response interface{}
			if err := msgpack.Unmarshal(result.Data(), &response); err != nil {
				return nil, err
			}

			if response != nil {
				return response, nil
			}
		}

		return nil, nil
	}
}
