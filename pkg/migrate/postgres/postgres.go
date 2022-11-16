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
	"database/sql"

	"github.com/go-logr/logr"
	"github.com/golang-migrate/migrate/v4"
	"github.com/golang-migrate/migrate/v4/database/postgres"
	_ "github.com/golang-migrate/migrate/v4/source/file"
	_ "github.com/lib/pq"

	"github.com/nanobus/nanobus/pkg/config"
	nb_migrate "github.com/nanobus/nanobus/pkg/migrate"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/runtime"
)

type Config struct {
	Name       string           `mapstructure:"name" validate:"required"`
	DataSource string           `mapstructure:"dataSource" validate:"required"`
	Directory  runtime.FilePath `mapstructure:"directory" validate:"required_without=SourceURL"`
	SourceURL  string           `mapstructure:"sourceUrl" validate:"required_without=Directory"`
}

func NamedLoader() (string, nb_migrate.Loader) {
	return "postgres", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (nb_migrate.Migrater, error) {
	var c Config
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

func Migrate(log logr.Logger, c *Config) nb_migrate.Migrater {
	return func(ctx context.Context) error {
		db, err := sql.Open("postgres", c.DataSource)
		if err != nil {
			return err
		}

		driver, err := postgres.WithInstance(db, &postgres.Config{})
		if err != nil {
			return err
		}

		sourceURL := c.SourceURL
		if c.Directory != "" {
			sourceURL = "file://" + c.Directory.Relative()
		}

		m, err := migrate.NewWithDatabaseInstance(
			sourceURL,
			"postgres", driver)
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
