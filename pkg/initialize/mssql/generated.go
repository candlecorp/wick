// Code generated by @apexlang/codegen. DO NOT EDIT.

package mssql

import (
	"github.com/nanobus/nanobus/pkg/initialize"
)

// This component offers database migrations for Postgres using the
// [golang-migrate/migrate](https://github.com/golang-migrate/migrate) library. It
// reads migrations from sources (`.sql` files with
// [DDL](https://en.wikipedia.org/wiki/Data_definition_language)) and applies them
// in correct order to a database.
type MigrateMSSQLV1Config struct {
	Name            string  `json:"name" yaml:"name" msgpack:"name" mapstructure:"name" validate:"required"`
	DataSource      string  `json:"dataSource" yaml:"dataSource" msgpack:"dataSource" mapstructure:"dataSource" validate:"required"`
	Directory       *string `json:"directory,omitempty" yaml:"directory,omitempty" msgpack:"directory,omitempty" mapstructure:"directory" validate:"required_without=SourceURL"`
	SourceURL       *string `json:"sourceUrl,omitempty" yaml:"sourceUrl,omitempty" msgpack:"sourceUrl,omitempty" mapstructure:"sourceUrl" validate:"required_without=Directory"`
	MigrationsTable *string `json:"migrationsTable,omitempty" yaml:"migrationsTable,omitempty" msgpack:"migrationsTable,omitempty" mapstructure:"migrationsTable"`
	DatabaseName    *string `json:"databaseName,omitempty" yaml:"databaseName,omitempty" msgpack:"databaseName,omitempty" mapstructure:"databaseName"`
	SchemaName      *string `json:"schemaName,omitempty" yaml:"schemaName,omitempty" msgpack:"schemaName,omitempty" mapstructure:"schemaName"`
}

func MigrateMSSQLV1() (string, initialize.Loader) {
	return "nanobus.migrate.mssql/v1", MigrateMSSQLV1Loader
}
