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
	"errors"
	"fmt"
	"reflect"

	"github.com/cenkalti/backoff"
	"github.com/go-redis/redis/v8"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func GetLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := GetConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}
	client, err := resource.Get[*redis.Client](resources, string(c.Resource))
	if err != nil {
		return nil, err
	}

	return GetAction(&c, client), nil
}

func GetAction(
	config *GetConfig,
	client *redis.Client) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		keyInt, err := config.Key.Eval(data)
		if err != nil {
			return nil, backoff.Permanent(fmt.Errorf("could not evaluate key: %w", err))
		}
		if isNil(keyInt) {
			return nil, backoff.Permanent(errors.New("key is nil"))
		}
		key := fmt.Sprintf("%v", keyInt)
		var result any
		err = client.Get(ctx, key).Scan(&result)

		// dataBytes, err := io.ReadAll(key)
		// if err != nil {
		// 	return nil, fmt.Errorf("could not read key: %w", err)
		// }

		// result, _, err := codec.Decode(dataBytes, config.CodecArgs...)
		return result, err
	}
}

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
}
