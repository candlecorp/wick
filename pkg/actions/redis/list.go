/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package redis

// import (
// 	"context"
// 	"fmt"

// 	"github.com/go-redis/redis/v8"

// 	"github.com/nanobus/nanobus/pkg/actions"
// 	"github.com/nanobus/nanobus/pkg/config"
// 	"github.com/nanobus/nanobus/pkg/resolve"
// 	"github.com/nanobus/nanobus/pkg/resource"
// )

// func ListLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
// 	c := ListConfig{}
// 	if err := config.Decode(with, &c); err != nil {
// 		return nil, err
// 	}

// 	var resources resource.Resources
// 	if err := resolve.Resolve(resolver,
// 		"resource:lookup", &resources); err != nil {
// 		return nil, err
// 	}

// 	poolI, ok := resources[string(c.Resource)]
// 	if !ok {
// 		return nil, fmt.Errorf("resource %q is not registered", c.Resource)
// 	}
// 	pool, ok := poolI.(*redis.Client)
// 	if !ok {
// 		return nil, fmt.Errorf("resource %q is not a *pgxpool.Pool", c.Resource)
// 	}

// 	return ListAction(&c, pool), nil
// }

// func ListAction(
// 	config *ListConfig,
// 	pool *redis.Client) actions.Action {
// 	return func(ctx context.Context, data actions.Data) (interface{}, error) {
// 		return pool.Keys(ctx, config.Keylist).Result()
// 	}
// }
