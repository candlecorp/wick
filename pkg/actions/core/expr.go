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

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/coalesce"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
)

// Backward compatability with `assign`
func Assign() (string, actions.Loader) {
	return "assign", ExprLoader
}

func ExprLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c ExprConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return EvalAction(&c), nil
}

func EvalAction(
	config *ExprConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (output interface{}, err error) {
		if config.Value != nil {
			output, err = config.Value.Eval(data)
			if err != nil {
				return nil, err
			}
		} else if config.Data != nil {
			output, err = config.Data.Eval(data)
			if err != nil {
				return nil, err
			}
			if v, ok := coalesce.ToMapSI(output, true); ok {
				output = v
			}
		}

		if config.To != nil {
			data[*config.To] = output
		}

		return output, nil
	}
}
