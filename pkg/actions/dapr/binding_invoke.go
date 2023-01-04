/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package dapr

import (
	"context"
	"fmt"

	dapr "github.com/dapr/go-sdk/client"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/coalesce"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func InvokeBindingLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := InvokeBindingConfig{
		Resource: "dapr",
		Codec:    "json",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	var codecs codec.Codecs
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources,
		"codec:lookup", &codecs); err != nil {
		return nil, err
	}

	codec, ok := codecs[string(c.Codec)]
	if !ok {
		return nil, fmt.Errorf("unknown codec %q", c.Codec)
	}

	client, err := resource.Get[dapr.Client](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return InvokeBindingAction(client, codec, &c), nil
}

func InvokeBindingAction(
	client dapr.Client,
	codec codec.Codec,
	config *InvokeBindingConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var bindingData interface{}
		r := dapr.InvokeBindingRequest{
			Name:      config.Binding,
			Operation: config.Operation,
		}

		var err error
		if config.Data != nil {
			if bindingData, err = config.Data.Eval(data); err != nil {
				return nil, err
			}
		}
		if config.Metadata != nil {
			if r.Metadata, err = config.Metadata.EvalMap(data); err != nil {
				return nil, err
			}
		}

		if r.Data, err = codec.Encode(bindingData, config.CodecArgs...); err != nil {
			return nil, err
		}

		resp, err := client.InvokeBinding(ctx, &r)
		if err != nil {
			return nil, err
		}

		var response interface{}
		if resp != nil && len(resp.Data) > 0 {
			err = coalesce.JSONUnmarshal(resp.Data, &response)
		}

		return response, err
	}
}
