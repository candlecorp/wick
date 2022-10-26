package postgres

import (
	"context"
	"database/sql"

	"github.com/golang-migrate/migrate/v4"
	"github.com/golang-migrate/migrate/v4/database/postgres"
	_ "github.com/golang-migrate/migrate/v4/source/file"
	_ "github.com/lib/pq"

	"github.com/nanobus/nanobus/config"
	nb_migrate "github.com/nanobus/nanobus/migrate"
	"github.com/nanobus/nanobus/resolve"
)

type Config struct {
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

	return Migrate(&c), nil
}

func Migrate(c *Config) nb_migrate.Migrater {
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

		return m.Up()
	}
}
