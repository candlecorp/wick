/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core

import (
	"context"
	"fmt"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/runtime"
)

type Condition struct {
	Name string
	When *expr.ValueExpr
	Then runtime.Runnable
}

func RouteLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c RouteConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var processor Processor
	if err := resolve.Resolve(resolver,
		"system:processor", &processor); err != nil {
		return nil, err
	}

	setups := make([]Condition, len(c.Routes))
	for i := range c.Routes {
		r := &c.Routes[i]

		runnable, err := processor.LoadPipeline(&runtime.Pipeline{
			Name:  r.Name,
			Steps: r.Then,
		})
		if err != nil {
			return nil, err
		}
		setups[i] = Condition{
			Name: r.Name,
			When: r.When,
			Then: runnable,
		}
	}

	return RouteAction(c.Selection, setups), nil
}

func RouteAction(
	selectionMode SelectionMode, routes []Condition) actions.Action {
	return func(ctx context.Context, data actions.Data) (output interface{}, err error) {
		for i := range routes {
			r := &routes[i]

			if r.When != nil {
				resultInt, err := r.When.Eval(data)
				if err != nil {
					return nil, err
				}

				result, ok := resultInt.(bool)
				if !ok {
					return nil, fmt.Errorf("expression %q did not evaluate a boolean", r.When.Expr())
				}

				if !result {
					continue
				}
			}

			output, err = r.Then(ctx, data)
			if selectionMode == SelectionModeSingle || err != nil {
				return output, err
			}
		}

		return output, nil
	}
}
