/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package postgres

import (
	"context"

	"github.com/jackc/pgx/v4/pgxpool"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type ConnectionConfig struct {
	URL string `mapstructure:"url"`
}

// Connection is the NamedLoader for a postgres connection.
func Connection() (string, resource.Loader) {
	return "postgres", ConnectionLoader
}

func ConnectionLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c ConnectionConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	config, err := pgxpool.ParseConfig(c.URL)
	if err != nil {
		return nil, err
	}
	// if len(afterConnect) > 0 {
	// 	config.AfterConnect = afterConnect[0]
	// }

	pool, err := pgxpool.ConnectConfig(ctx, config)
	if err != nil {
		return nil, err
	}

	return pool, pool.Ping(ctx)
}
