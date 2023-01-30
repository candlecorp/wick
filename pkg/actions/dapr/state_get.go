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
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func GetStateLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := GetStateConfig{
		Resource:      "dapr",
		Codec:         "json",
		NotFoundError: "not_found",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}
	if c.CodecArgs == nil {
		c.CodecArgs = []any{}
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
		return nil, fmt.Errorf("codec %q not found", c.Codec)
	}

	client, err := resource.Get[dapr.Client](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return GetStateAction(client, codec, &c), nil
}

func GetStateAction(
	client dapr.Client,
	codec codec.Codec,
	config *GetStateConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		key, err := expr.EvalAsStringE(config.Key, data)
		if err != nil {
			return nil, fmt.Errorf("could not evaluate key: %w", err)
		}

		resp, err := client.GetStateWithConsistency(ctx, config.Store,
			key, nil, dapr.StateConsistency(config.Consistency))
		if err != nil {
			return nil, resiliency.Retriable(err)
		}

		var response interface{}
		if resp != nil && len(resp.Value) > 0 {
			response, _, err = codec.Decode(resp.Value, config.CodecArgs...)
		} else if config.NotFoundError != "" {
			return nil, errorz.Return(config.NotFoundError, errorz.Metadata{
				"resource": config.Resource,
				"key":      key,
			})
		}

		return response, err
	}
}
