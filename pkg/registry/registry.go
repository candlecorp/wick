/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package registry

import (
	"context"

	"github.com/nanobus/nanobus/pkg/resolve"
)

type (
	NamedLoader[T any] func() (string, Loader[T])
	Loader[T any]      func(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (T, error)
	Registry[T any]    map[string]Loader[T]
)

func (r Registry[T]) Register(loaders ...NamedLoader[T]) {
	for _, l := range loaders {
		name, loader := l()
		r[name] = loader
	}
}
