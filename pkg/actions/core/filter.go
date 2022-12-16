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
	"github.com/nanobus/nanobus/pkg/resolve"
)

func FilterLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c FilterConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return FilterAction(&c), nil
}

func FilterAction(
	config *FilterConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		resultInt, err := config.Condition.Eval(data)
		if err != nil {
			return nil, err
		}

		result, ok := resultInt.(bool)
		if !ok {
			return nil, fmt.Errorf("expression %q did not evaluate a boolean", config.Condition.Expr())
		}

		if !result {
			return nil, actions.Stop()
		}

		return nil, nil
	}
}
