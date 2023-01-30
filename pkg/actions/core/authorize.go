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
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/security/claims"
)

func AuthorizeLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := AuthorizeConfig{
		Error: "permission_denied",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return AuthorizeAction(&c), nil
}

func AuthorizeAction(
	config *AuthorizeConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		if config.Condition != nil {
			result, err := expr.EvalAsBoolE(config.Condition, data)
			if err != nil {
				return nil, fmt.Errorf("expression %q did not evaluate a boolean: %w", config.Condition.Expr(), err)
			}

			if !result {
				return nil, errorz.Return(config.Error, errorz.Metadata{
					"expr": config.Condition.Expr(),
				})
			}
		}

		claimsMap := claims.FromContext(ctx)

		for _, claim := range config.Has {
			if _, ok := claimsMap[claim]; !ok {
				return nil, errorz.Return(config.Error, errorz.Metadata{
					"claim": claim,
				})
			}
		}

		for claim, value := range config.Check {
			v := claimsMap[claim]
			if v != value {
				return nil, errorz.Return(config.Error, errorz.Metadata{
					"claim": claim,
					"want":  value,
				})
			}
		}

		return nil, nil
	}
}
