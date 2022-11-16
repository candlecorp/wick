/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package filter

import (
	"context"

	"github.com/nanobus/nanobus/pkg/registry"
)

type (
	NamedLoader = registry.NamedLoader[Filter]
	Loader      = registry.Loader[Filter]
	Registry    = registry.Registry[Filter]

	Filter func(ctx context.Context, header Header) (context.Context, error)

	Header interface {
		Get(name string) string
		Values(name string) []string
		Set(name, value string)
	}
)
