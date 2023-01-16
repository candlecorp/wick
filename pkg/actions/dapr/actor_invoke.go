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

	"github.com/cenkalti/backoff/v4"
	dapr "github.com/dapr/go-sdk/client"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func InvokeActorLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := InvokeActorConfig{
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

	return InvokeActorAction(client, codec, &c), nil
}

func InvokeActorAction(
	client dapr.Client,
	codec codec.Codec,
	config *InvokeActorConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var inputData interface{}

		idInt, err := config.ID.Eval(data)
		if err != nil {
			return nil, backoff.Permanent(fmt.Errorf("could not evaluate id: %w", err))
		}

		r := dapr.InvokeActorRequest{
			ActorType: config.Handler.Interface,
			ActorID:   fmt.Sprintf("%v", idInt),
			Method:    config.Handler.Operation,
		}

		if config.Data != nil {
			if inputData, err = config.Data.Eval(data); err != nil {
				return nil, err
			}
		} else {
			inputData = data["input"]
		}

		if r.Data, err = codec.Encode(inputData, config.CodecArgs...); err != nil {
			return nil, err
		}

		resp, err := client.InvokeActor(ctx, &r)
		if err != nil {
			return nil, err
		}

		var response interface{}
		if resp != nil && len(resp.Data) > 0 {
			response, _, err = codec.Decode(resp.Data)
		}

		return response, err
	}
}
