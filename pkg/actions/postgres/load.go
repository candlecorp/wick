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

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/spec"
)

// Load is the NamedLoader for the invoke action.
func LoadLeader() (string, actions.Loader) {
	return "@postgres/load", LoadLoader
}

func LoadLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := LoadConfig{
		NotFoundError: "not_found",
	}
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

	ns, ok := namespaces[c.Entity.Namespace]
	if !ok {
		return nil, fmt.Errorf("namespace %q is not found", c.Entity.Namespace)
	}
	t, ok := ns.Type(c.Entity.Type)
	if !ok {
		return nil, fmt.Errorf("type %q is not found", c.Entity.Type)
	}

	return LoadAction(&c, t, ns, pool), nil
}

func LoadAction(
	config *LoadConfig,
	t *spec.Type,
	ns *spec.Namespace,
	pool *pgxpool.Pool) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		keyValue, err := config.Key.Eval(data)
		if err != nil {
			return nil, err
		}

		var result map[string]interface{}
		err = pool.AcquireFunc(ctx, func(conn *pgxpool.Conn) (err error) {
			result, err = findById(ctx, conn, t, keyValue, config.Preload)
			return err
		})

		if result == nil && config.NotFoundError != "" {
			return nil, errorz.Return(config.NotFoundError, errorz.Metadata{
				"resource": config.Resource,
				"key":      keyValue,
			})
		}

		return result, err
	}
}
