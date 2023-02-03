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
	"fmt"

	"github.com/jmoiron/sqlx"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func ExecMultiLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := ExecMultiConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	db, err := resource.Get[*sqlx.DB](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return ExecMultiAction(db, &c), nil
}

func ExecMultiAction(
	db *sqlx.DB,
	config *ExecMultiConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		tx, err := db.Begin()
		if err != nil {
			return nil, fmt.Errorf("could not start transaction: %w", err)
		}
		defer tx.Rollback()

		for _, stmt := range config.Statements {
			var err error
			var input interface{} = map[string]interface{}(data)
			if stmt.Data != nil {
				input, err = stmt.Data.Eval(data)
				if err != nil {
					return nil, err
				}
			}

			if multi, ok := input.([]interface{}); ok {
				for _, item := range multi {
					if single, ok := item.(map[string]interface{}); ok {
						single["$root"] = data
						args := make([]interface{}, len(stmt.Args))
						for i, expr := range stmt.Args {
							var err error
							if args[i], err = expr.Eval(single); err != nil {
								delete(single, "$root")
								return nil, err
							}
						}

						_, err := tx.Exec(stmt.SQL, args...)
						if err != nil {
							delete(single, "$root")
							return nil, err
						}
						// if tag.RowsAffected() == 0 {
						// 	delete(single, "$root")
						// 	return errors.New("no rows effected")
						// }
						delete(single, "$root")
					}
				}
			} else if single, ok := input.(map[string]interface{}); ok {
				single["$root"] = data
				args := make([]interface{}, len(stmt.Args))
				for i, expr := range stmt.Args {
					var err error
					if args[i], err = expr.Eval(single); err != nil {
						delete(single, "$root")
						return nil, err
					}
				}

				_, err := tx.Exec(stmt.SQL, args...)
				if err != nil {
					delete(single, "$root")
					return nil, err
				}
				// if tag.RowsAffected() == 0 {
				// 	delete(single, "$root")
				// 	return errors.New("no rows effected")
				// }
				delete(single, "$root")
			}
		}

		return nil, tx.Commit()
	}
}
