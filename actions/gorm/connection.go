package gorm

import (
	"context"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"

	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/resource"
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
