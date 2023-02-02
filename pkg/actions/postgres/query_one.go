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
	"strings"

	"github.com/jackc/pgx/v4/pgxpool"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func QueryOneLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := QueryOneConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}
	c.SQL = strings.TrimSpace(c.SQL)

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	pool, err := resource.Get[*pgxpool.Pool](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	// Return QueryAction with Single = true.
	return QueryAction(&QueryConfig{
		Resource: c.Resource,
		SQL:      c.SQL,
		Args:     c.Args,
		Single:   true,
	}, pool), nil
}
