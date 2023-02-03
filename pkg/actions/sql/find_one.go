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
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/spec"
)

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

	db, err := resource.Get[*sqlx.DB](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	ns, ok := namespaces[c.Entity.Namespace]
	if !ok {
		return nil, fmt.Errorf("namespace %q is not found", c.Entity.Namespace)
	}
	t, ok := ns.Type(c.Entity.Type)
	if !ok {
		return nil, fmt.Errorf("type %q is not found", c.Entity.Type)
	}

	return FindOneAction(&c, t, ns, db), nil
}

func FindOneAction(
	config *FindOneConfig,
	t *spec.Type,
	ns *spec.Namespace,
	db *sqlx.DB) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		result, err := findOne(ctx, db, t, data, config.Where, config.Preload)
		if err != nil {
			return nil, err
		}

		if result == nil && config.NotFoundError != nil {
			return nil, errorz.Return(*config.NotFoundError, errorz.Metadata{
				"resource": config.Resource,
			})
		}

		return result, nil
	}
}
