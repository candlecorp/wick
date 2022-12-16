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

	return CallProviderAction(c.Handler.Interface, c.Handler.Operation, processor), nil
}

func CallProviderAction(
	name, operation string, processor Processor) actions.Action {
	return func(ctx context.Context, data actions.Data) (output interface{}, err error) {
		var ok bool
		output, ok, err = processor.Provider(ctx, name, operation, data)
		if !ok {
			return nil, fmt.Errorf("could not find provider %s::%s", name, operation)
		}
		return
	}
}
