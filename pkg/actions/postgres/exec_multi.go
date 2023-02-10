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

	"github.com/jackc/pgx/v4"
	"github.com/jackc/pgx/v4/pgxpool"

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

	poolI, ok := resources[string(c.Resource)]
	if !ok {
		return nil, fmt.Errorf("resource %q is not registered", c.Resource)
	}
	pool, ok := poolI.(*pgxpool.Pool)
	if !ok {
		return nil, fmt.Errorf("resource %q is not a *pgxpool.Pool", c.Resource)
	}

	return ExecMultiAction(&c, pool), nil
}

func ExecMultiAction(
	config *ExecMultiConfig,
	pool *pgxpool.Pool) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		err := pool.BeginFunc(ctx, func(tx pgx.Tx) error {
			for _, stmt := range config.Statements {
				var err error
				var input interface{} = map[string]interface{}(data)
				if stmt.Data != nil {
					input, err = stmt.Data.Eval(data)
					if err != nil {
						return err
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
									return err
								}
							}

							_, err := tx.Exec(ctx, stmt.SQL, args...)
							if err != nil {
								delete(single, "$root")
								return err
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
							return err
						}
					}

					_, err := tx.Exec(ctx, stmt.SQL, args...)
					if err != nil {
						delete(single, "$root")
						return err
					}
					// if tag.RowsAffected() == 0 {
					// 	delete(single, "$root")
					// 	return errors.New("no rows effected")
					// }
					delete(single, "$root")
				}
			}
			return nil
		})

		return nil, err
	}
}
