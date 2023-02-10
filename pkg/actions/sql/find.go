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
	"github.com/spf13/cast"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/spec"
	"github.com/nanobus/nanobus/pkg/stream"
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

	db, err := resource.Get[*sqlx.DB](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	ns, ok := namespaces[string(c.Entity.Namespace)]
	if !ok {
		return nil, fmt.Errorf("namespace %q is not found", c.Entity.Namespace)
	}
	t, ok := ns.Type(c.Entity.Type)
	if !ok {
		return nil, fmt.Errorf("type %q is not found", c.Entity.Type)
	}

	return FindAction(&c, t, ns, db), nil
}

func FindAction(
	config *FindConfig,
	t *spec.Type,
	ns *spec.Namespace,
	db *sqlx.DB) actions.Action {
	return func(ctx context.Context, data actions.Data) (_ interface{}, err error) {
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
			limit, err = expr.EvalAsInt64E(config.Limit, data)
			if err != nil {
				return nil, err
			}
		}

		s, _ := stream.SinkFromContext(ctx)

		if s != nil {
			if err := streamMany(ctx, s, db, t, data, config.Where, config.Preload, offset, limit); err != nil {
				return nil, err
			}

			return nil, nil
		} else {
			if config.Pagination != nil {
				total, err = getCount(ctx, db, t, data, config.Where)
				if err != nil {
					return nil, err
				}
			}
			results, err = getMany(ctx, db, t, data, config.Where, config.Preload, offset, limit)
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
}
