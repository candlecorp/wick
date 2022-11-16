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

	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type LogConfig struct {
	Format string `mapstructure:"format" validate:"required"`
	// Args are the evaluations to use as arguments into the string format.
	Args []*expr.ValueExpr `mapstructure:"args"`
}

type Logger interface {
	Printf(format string, v ...interface{})
}

// Log is the NamedLoader for the log action.
func Log() (string, actions.Loader) {
	return "log", LogLoader
}

func LogLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c LogConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var logger logr.Logger
	if err := resolve.Resolve(resolver,
		"system:logger", &logger); err != nil {
		return nil, err
	}

	return LogAction(logger, &c), nil
}

func LogAction(
	logger logr.Logger,
	config *LogConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		args := make([]interface{}, len(config.Args))
		for i, expr := range config.Args {
			var err error
			if args[i], err = expr.Eval(data); err != nil {
				return nil, err
			}
		}

		msg := config.Format
		if len(args) > 0 {
			msg = fmt.Sprintf(msg, args...)
		}

		logger.Info(msg)

		return nil, nil
	}
}
