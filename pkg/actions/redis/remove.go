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
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func RemoveLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := RemoveConfig{
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

	return RemoveAction(&c, codec, client), nil
}

func RemoveAction(
	config *RemoveConfig,
	codec codec.Codec,
	client *redis.Client) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		key, err := expr.EvalAsStringE(config.Key, data)
		if err != nil {
			return nil, fmt.Errorf("could not evaluate key: %w", err)
		}
		return client.Del(ctx, key).Result()
	}
}
