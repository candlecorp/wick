/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package transport

import (
	"context"
	"errors"

	"github.com/nanobus/nanobus/pkg/resolve"
)

var ErrBadInput = errors.New("input was malformed")

type (
	NamedLoader func() (string, Loader)
	Loader      func(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (Transport, error)

	Transport interface {
		Listen() error
		Close() error
	}

	Invoker func(ctx context.Context, iface, id, operation string, input interface{}) (interface{}, error)

	Registry map[string]Loader
)

func (r Registry) Register(loaders ...NamedLoader) {
	for _, l := range loaders {
		name, loader := l()
		r[name] = loader
	}
}
