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
	"errors"
	"fmt"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/stream"
)

type TestConfig struct {
	Data *expr.DataExpr `mapstructure:"data"`
}

// Test is the NamedLoader for the log action.
func Test() (string, actions.Loader) {
	return "@postgres/test", TestLoader
}

func TestLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c TestConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return TestAction(&c), nil
}

func TestAction(
	config *TestConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		s, ok := stream.SinkFromContext(ctx)
		if !ok {
			return nil, errors.New("stream not in context")
		}

		v, err := config.Data.Eval(data)
		if err != nil {
			fmt.Println(err)
			return nil, err
		}

		for i := 0; i < 10; i++ {
			if err = s.Next(v, nil); err != nil {
				fmt.Println(err)
				return nil, err
			}
		}

		return nil, nil
	}
}
