/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package redis

import (
	"context"
	"fmt"

	"github.com/go-redis/redis/v8"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func SetLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := SetConfig{
		Codec: "bytes",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	var codecs codec.Codecs
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	codec, ok := codecs[string(c.Codec)]
	if !ok {
		return nil, fmt.Errorf("unknown codec %q", c.Codec)
	}

	client, err := resource.Get[*redis.Client](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return SetAction(&c, codec, client), nil
}

func SetAction(
	config *SetConfig,
	codec codec.Codec,
	client *redis.Client) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		key, err := expr.EvalAsStringE(config.Key, data)
		if err != nil {
			return nil, fmt.Errorf("could not evaluate key: %w", err)
		}

		value, err := expr.EvalAsStringE(config.Data, data)
		if err != nil {
			return nil, fmt.Errorf("could not evaluate value: %w", err)
		}

		dataBytes, err := client.Set(ctx, key, value, 0).Result()
		if err != nil {
			return nil, resiliency.Retriable(fmt.Errorf("could not read key: %w", err))
		}

		ans := []byte(dataBytes)

		result, _, err := codec.Decode(ans, config.CodecArgs...)

		return result, err
	}
}
