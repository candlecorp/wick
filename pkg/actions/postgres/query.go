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
	"fmt"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v4"
	"github.com/jackc/pgx/v4/pgxpool"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/stream"
)

func QueryLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := QueryConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}
	c.SQL = strings.TrimSpace(c.SQL)

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	poolI, ok := resources[string(c.Resource)]
	if !ok {
		return nil, fmt.Errorf("resource %q is not registered", c.Resource)
	}
	pool, ok := poolI.(*pgxpool.Pool)
	if !ok {
		return nil, fmt.Errorf("resource %q is not a *pgxpool.Pool", c.Resource)
	}

	return QueryAction(&c, pool), nil
}

func QueryAction(
	config *QueryConfig,
	pool *pgxpool.Pool) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		s, _ := stream.SinkFromContext(ctx)

		args := make([]interface{}, len(config.Args))
		for i, expr := range config.Args {
			var err error
			if args[i], err = expr.Eval(data); err != nil {
				return nil, err
			}
		}

		rows, err := pool.Query(ctx, config.SQL, args...)
		if err != nil {
			return nil, err
		}
		defer rows.Close()

		fields := rows.FieldDescriptions()
		fieldNames := make([]string, len(fields))
		for i, f := range fields {
			fieldNames[i] = snakeCaseToCamelCase(string(f.Name))
		}

		if config.Single {
			if rows.Next() {
				record, err := rowsToRecord(rows, fieldNames)
				if err != nil {
					return nil, err
				}

				return record, nil
			}
		} else {
			if s != nil {
				for rows.Next() {
					record, err := rowsToRecord(rows, fieldNames)
					if err != nil {
						return nil, err
					}

					if err = s.Next(record, nil); err != nil {
						return nil, err
					}
				}
			} else {
				records := make([]any, 0, 20)
				for rows.Next() {
					record, err := rowsToRecord(rows, fieldNames)
					if err != nil {
						return nil, err
					}

					records = append(records, record)
				}

				return records, nil
			}
		}

		return nil, nil
	}
}

func rowsToRecord(rows pgx.Rows, fieldNames []string) (any, error) {
	record := make(map[string]interface{})
	values, err := rows.Values()
	if err != nil {
		return nil, err
	}
	for i, v := range values {
		switch vv := v.(type) {
		// Assume [16]byte are UUID types in Postgres
		// and convert to string
		case [16]byte:
			v = uuid.UUID(vv).String()
		}
		record[fieldNames[i]] = v
	}

	return record, nil
}

func snakeCaseToCamelCase(inputUnderScoreStr string) (camelCase string) {
	isToUpper := false
	for k, v := range inputUnderScoreStr {
		if k == 0 {
			camelCase = strings.ToLower(string(inputUnderScoreStr[0]))
		} else {
			if isToUpper {
				camelCase += strings.ToUpper(string(v))
				isToUpper = false
			} else {
				if v == '_' {
					isToUpper = true
				} else {
					camelCase += string(v)
				}
			}
		}
	}
	return
}
