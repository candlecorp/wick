/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package gorm

import (
	"context"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type ConnectionConfig struct {
	DSN string `mapstructure:"dsn"`
}

// Connection is the NamedLoader for a postgres connection.
func Connection() (string, resource.Loader) {
	return "gorm:postgres", ConnectionLoader
}

func ConnectionLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c ConnectionConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	db, err := gorm.Open(postgres.New(postgres.Config{
		DSN: c.DSN,
		// disables implicit prepared statement usage. By default pgx automatically uses the extended protocol
		PreferSimpleProtocol: true,
	}), &gorm.Config{})

	return db, err
}
