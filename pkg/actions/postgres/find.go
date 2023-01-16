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

	"github.com/jackc/pgx/v4/pgxpool"
	"github.com/spf13/cast"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/spec"
)

func FindLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := FindConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var namespaces spec.Namespaces
	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"spec:namespaces", &namespaces,
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

	ns, ok := namespaces[string(c.Entity.Namespace)]
	if !ok {
		return nil, fmt.Errorf("namespace %q is not found", c.Entity.Namespace)
	}
	t, ok := ns.Type(c.Entity.Type)
	if !ok {
		return nil, fmt.Errorf("type %q is not found", c.Entity.Type)
	}

	return FindAction(&c, t, ns, pool), nil
}

func FindAction(
	config *FindConfig,
	t *spec.Type,
	ns *spec.Namespace,
	pool *pgxpool.Pool) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var results []map[string]interface{}
		var total int64
		offset := int64(0)
		limit := int64(1000)

		if config.Offset != nil {
			v, err := config.Offset.Eval(data)
			if err != nil {
				return nil, err
			}
			offset, err = cast.ToInt64E(v)
			if err != nil {
				return nil, err
			}
		}

		if config.Limit != nil {
			v, err := config.Limit.Eval(data)
			if err != nil {
				return nil, err
			}
			limit, err = cast.ToInt64E(v)
			if err != nil {
				return nil, err
			}
		}

		err := pool.AcquireFunc(ctx, func(conn *pgxpool.Conn) (err error) {
			if config.Pagination != nil {
				total, err = getCount(ctx, conn, t, data, config.Where)
				if err != nil {
					return err
				}
			}
			results, err = getMany(ctx, conn, t, data, config.Where, config.Preload, offset, limit)
			return err
		})
		if err != nil {
			return nil, err
		}

		if config.Pagination != nil {
			p := config.Pagination
			count := int64(len(results))
			wrapper := map[string]interface{}{
				p.Items: results,
			}
			if p.Total != nil {
				wrapper[*p.Total] = total
			}
			if p.Count != nil {
				wrapper[*p.Count] = count
			}
			if p.PageCount != nil {
				wrapper[*p.PageCount] = (count + limit - 1) / limit
			}
			if p.PageIndex != nil {
				wrapper[*p.PageIndex] = offset / limit
			}
			if p.Offset != nil {
				wrapper[*p.Offset] = offset
			}
			if p.Limit != nil {
				wrapper[*p.Limit] = config.Limit
			}

			return wrapper, nil
		}

		return results, nil
	}
}
