/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package mssql

import (
	"context"
	"database/sql"

	"github.com/go-logr/logr"
	"github.com/golang-migrate/migrate/v4"
	"github.com/golang-migrate/migrate/v4/database/sqlserver"
	_ "github.com/golang-migrate/migrate/v4/source/file"
	_ "github.com/microsoft/go-mssqldb"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/initialize"
	"github.com/nanobus/nanobus/pkg/resolve"
)

func MigrateMSSQLV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (initialize.Initializer, error) {
	var c MigrateMSSQLV1Config
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var logger logr.Logger
	if err := resolve.Resolve(resolver,
		"system:logger", &logger); err != nil {
		return nil, err
	}

	return Migrate(logger, &c), nil
}

func Migrate(log logr.Logger, c *MigrateMSSQLV1Config) initialize.Initializer {
	return func(ctx context.Context) error {
		db, err := sql.Open("sqlserver", c.DataSource)
		if err != nil {
			return err
		}

		driver, err := sqlserver.WithInstance(db, &sqlserver.Config{
			MigrationsTable: orEmptyValue(c.MigrationsTable),
			DatabaseName:    orEmptyValue(c.DatabaseName),
			SchemaName:      orEmptyValue(c.SchemaName),
		})
		if err != nil {
			return err
		}

		sourceURL := ""
		if c.SourceURL != nil {
			sourceURL = *c.SourceURL
		} else if c.Directory != nil {
			sourceURL = "file://" + *c.Directory
		}

		m, err := migrate.NewWithDatabaseInstance(
			sourceURL,
			"sqlserver", driver)
		if err != nil {
			return err
		}

		if err := m.Up(); err != nil {
			if err != migrate.ErrNoChange {
				return err
			}

			log.Info("Migration has no changes", "name", c.Name)
		} else {
			log.Info("Migration successful", "name", c.Name)
		}

		return nil
	}
}

type Ptr[T any] interface {
	*T
}

func orEmptyValue[T any, P Ptr[T]](val P) (ret T) {
	if val != nil {
		ret = *val
	}
	return ret
}
