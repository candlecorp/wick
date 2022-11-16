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
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/spec"
)

type FindConfig struct {
	// Resource is the name of the connection resource to use.
	Resource string `mapstructure:"resource" validate:"required"`
	// Namespace is the type namespace to load.
	Namespace string `mapstructure:"namespace" validate:"required"`
	// Type is the type name to load.
	Type string `mapstructure:"type" validate:"required"`
	// Preload lists the relationship to expand/load.
	Preload []Preload `mapstructure:"preload"`
	// Where list the parts of the where clause.
	Where []Where `mapstructure:"where"`
	// Pagination is the optional fields to wrap the results with.
	Pagination *Pagination `mapstructure:"pagination"`
	// Offset is the query offset.
	Offset *expr.ValueExpr `mapstructure:"offset"`
	// Limit is the query limit.
	Limit *expr.ValueExpr `mapstructure:"limit"`
}

type Pagination struct {
	PageIndex string `mapstructure:"pageIndex"`
	PageCount string `mapstructure:"pageCount"`
	Offset    string `mapstructure:"offset"`
	Limit     string `mapstructure:"limit"`
	Count     string `mapstructure:"count"`
	Total     string `mapstructure:"total"`
	Items     string `mapstructure:"items"`
}

// Find is the NamedLoader for the invoke action.
func Find() (string, actions.Loader) {
	return "@postgres/find", FindLoader
}

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

	poolI, ok := resources[c.Resource]
	if !ok {
		return nil, fmt.Errorf("resource %q is not registered", c.Resource)
	}
	pool, ok := poolI.(*pgxpool.Pool)
	if !ok {
		return nil, fmt.Errorf("resource %q is not a *pgxpool.Pool", c.Resource)
	}

	ns, ok := namespaces[c.Namespace]
	if !ok {
		return nil, fmt.Errorf("namespace %q is not found", c.Namespace)
	}
	t, ok := ns.Type(c.Type)
	if !ok {
		return nil, fmt.Errorf("type %q is not found", c.Type)
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
			if p.Total != "" {
				wrapper[p.Total] = total
			}
			if p.Count != "" {
				wrapper[p.Count] = count
			}
			if p.PageCount != "" {
				wrapper[p.PageCount] = (count + limit - 1) / limit
			}
			if p.PageIndex != "" {
				wrapper[p.PageIndex] = offset / limit
			}
			if p.Offset != "" {
				wrapper[p.Offset] = offset
			}
			if p.Limit != "" {
				wrapper[p.Limit] = config.Limit
			}

			return wrapper, nil
		}

		return results, nil
	}
}
