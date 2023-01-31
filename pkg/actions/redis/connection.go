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

	"github.com/nanobus/nanobus/pkg/logger"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type ConnectionConfig struct {
	URL string `mapstructure:"url"`
}

// Connection is the NamedLoader for a redis connection.
func Connection() (string, resource.Loader) {
	logger.Debug("Testing Golang Redis")
	return "redis", ConnectionLoader
}

func ConnectionLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {

	client := redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "",
		DB:       0,
	})

	pong, err := client.Ping(client.Context()).Result()
	fmt.Println(pong, err)

	return client, err
}
