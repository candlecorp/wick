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
	"strings"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/runtime"
)

// SelectionMode indicates how many routes can be selected.
type SelectionMode int

const (
	// Single indicates only one route can be selected.
	Single SelectionMode = iota
	// Multi indicates many routes can be selected.
	Multi
)

type RouteConfig struct {
	// Selection defines the selection mode: single or multi.
	Selection SelectionMode `mapstructure:"selection"`
	// Routes are the possible runnable routes which conditions for selection.
	Routes []RouteCondition `mapstructure:"routes"`
}

type RouteCondition struct {
	// Name if the overall summary of this route.
	Name string `mapstructure:"name"`
	// When is the predicate expression for filtering.
	When *expr.ValueExpr `mapstructure:"when"`
	// Then is the steps to process.
	Then []runtime.Step `mapstructure:"then"`
	// Call is the name of the pipeline to call.
	Call string `mapstructure:"call"`

	runnable runtime.Runnable
}

// Route is the NamedLoader for the filter action.
func Route() (string, actions.Loader) {
	return "route", RouteLoader
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

	for i := range c.Routes {
		r := &c.Routes[i]
		if r.Call != "" {
			r.runnable = func(ctx context.Context, data actions.Data) (interface{}, error) {
				return processor.Pipeline(ctx, r.Call, data)
			}
			continue
		}

		runnable, err := processor.LoadPipeline(&runtime.Pipeline{
			Name:  r.Name,
			Steps: r.Then,
		})
		if err != nil {
			return nil, err
		}
		r.runnable = runnable
	}

	return RouteAction(&c), nil
}

func RouteAction(
	config *RouteConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (output interface{}, err error) {
		for i := range config.Routes {
			r := &config.Routes[i]

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

			output, err = r.runnable(ctx, data)
			if config.Selection == Single || err != nil {
				return output, err
			}
		}

		return output, nil
	}
}

// DecodeString handles converting a string value to SelectionMode.
func (sm *SelectionMode) DecodeString(value string) error {
	switch strings.ToLower(value) {
	case "single":
		*sm = Single
	case "multi":
		*sm = Multi
	default:
		return fmt.Errorf("unexpected selection mode: %s", value)
	}

	return nil
}
