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

type FindOneConfig struct {
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
	// NotFoundError is the error to return if the key is not found.
	NotFoundError string `mapstructure:"notFoundError"`
}

// FindOne is the NamedLoader for the invoke action.
func FindOne() (string, actions.Loader) {
	return "@postgres/find_one", FindOneLoader
}

func FindOneLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := FindOneConfig{}
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

	return FindOneAction(&c, t, ns, pool), nil
}

func FindOneAction(
	config *FindOneConfig,
	t *spec.Type,
	ns *spec.Namespace,
	pool *pgxpool.Pool) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var result map[string]interface{}
		err := pool.AcquireFunc(ctx, func(conn *pgxpool.Conn) (err error) {
			result, err = findOne(ctx, conn, t, data, config.Where, config.Preload)
			return err
		})
		if err != nil {
			return nil, err
		}

		if result == nil && config.NotFoundError != "" {
			return nil, errorz.Return(config.NotFoundError, errorz.Metadata{
				"resource": config.Resource,
			})
		}

		return result, nil
	}
}
