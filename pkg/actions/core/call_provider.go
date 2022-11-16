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
	"errors"
	"strings"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type CallProviderConfig struct {
	// Namespace is the namespace of the provider to call.
	Namespace string `mapstructure:"namespace" validate:"required"`
	// Operation is the operation name of the provider to call.
	Operation string `mapstructure:"operation" validate:"required"`
}

// Route is the NamedLoader for the filter action.
func CallProvider() (string, actions.Loader) {
	return "call_provider", CallProviderLoader
}

func CallProviderLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c CallProviderConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var processor Processor
	if err := resolve.Resolve(resolver,
		"system:processor", &processor); err != nil {
		return nil, err
	}

	namespace := c.Namespace
	i := strings.LastIndex(namespace, ".")
	if i < 0 {
		return nil, errors.New("invalid namespace")
	}
	service := namespace[i+1:]
	namespace = namespace[:i]

	return CallProviderAction(namespace, service, c.Operation, processor), nil
}

func CallProviderAction(
	namespace, service, operation string, processor Processor) actions.Action {
	return func(ctx context.Context, data actions.Data) (output interface{}, err error) {
		return processor.Provider(ctx, namespace, service, operation, data)
	}
}
