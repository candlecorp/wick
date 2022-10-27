package postgres

import (
	"context"
	"database/sql"

	"github.com/go-logr/logr"
	"github.com/golang-migrate/migrate/v4"
	"github.com/golang-migrate/migrate/v4/database/postgres"
	_ "github.com/golang-migrate/migrate/v4/source/file"
	_ "github.com/lib/pq"

	"github.com/nanobus/nanobus/config"
	nb_migrate "github.com/nanobus/nanobus/migrate"
	"github.com/nanobus/nanobus/resolve"
)

type Config struct {
	Name       string `mapstructure:"name" validate:"required"`
	DataSource string `mapstructure:"dataSource" validate:"required"`
	SourceURL  string `mapstructure:"sourceUrl" validate:"required"`
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

		m, err := migrate.NewWithDatabaseInstance(
			c.SourceURL,
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
