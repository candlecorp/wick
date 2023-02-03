/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package sql

import (
	"context"
	"log"

	"github.com/jmoiron/sqlx"

	_ "github.com/go-sql-driver/mysql"          // MySQL
	_ "github.com/lib/pq"                       // Postgres
	_ "github.com/microsoft/go-mssqldb"         // MS SQL Server
	_ "github.com/microsoft/go-mssqldb/azuread" // Azure AD driver module
	_ "github.com/sijms/go-ora/v2"              // Oracle
	_ "github.com/snowflakedb/gosnowflake"      // Snowflake

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type ConnectionConfig struct {
	Driver     string `mapstructure:"driver"`
	DataSource string `mapstructure:"dataSource"`
}

// Connection is the NamedLoader for a postgres connection.
func Connection() (string, resource.Loader) {
	return "nanobus.resource.sql/v1", ConnectionLoader
}

func ConnectionLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c ConnectionConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	// This Pings the database trying to connect
	// use sqlx.Open() for sql.Open() semantics
	db, err := sqlx.Connect(c.Driver, c.DataSource)
	if err != nil {
		log.Fatalln(err)
	}

	return db, nil
}
