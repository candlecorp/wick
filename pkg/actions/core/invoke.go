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

	"github.com/nanobus/iota/go/wasmrs/payload"
	"github.com/nanobus/iota/go/wasmrs/rx/mono"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/function"
	"github.com/nanobus/nanobus/pkg/mesh"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type InvokeConfig struct {
	// Namespace of the service to invoke.
	Namespace string `mapstructure:"namespace"`
	// Operation of the service to invoke.
	Operation string `mapstructure:"operation"`
	// Input optionally transforms the input sent to the function.
	Input *expr.DataExpr `mapstructure:"input"`
}

type Invoker interface {
	RequestResponse(ctx context.Context, namespace, operation string, p payload.Payload) mono.Mono[payload.Payload]
}

// Invoke is the NamedLoader for the invoke action.
func Invoke() (string, actions.Loader) {
	return "invoke", InvokeLoader
}

func InvokeLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := InvokeConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var m *mesh.Mesh
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

		namespace := config.Namespace
		operation := config.Operation
		namespaceEmpty := namespace == ""
		operationEmpty := operation == ""

		// Grab the incoming function details if needed.
		if namespaceEmpty || operationEmpty {
			fn := function.FromContext(ctx)

			if namespaceEmpty {
				namespace = fn.Namespace
			}
			if operationEmpty {
				operation = fn.Operation
			}
		}

		payloadData, err := msgpack.Marshal(input)
		if err != nil {
			return nil, err
		}

		metadata := make([]byte, 8)
		p := payload.New(payloadData, metadata)

		result, err := invoker.RequestResponse(ctx, namespace, operation, p).Block()
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
