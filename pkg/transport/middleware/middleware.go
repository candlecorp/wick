/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package middleware

import (
	"net/http"

	"github.com/nanobus/nanobus/pkg/registry"
)

type (
	NamedLoader = registry.NamedLoader[Middleware]
	Loader      = registry.Loader[Middleware]
	Registry    = registry.Registry[Middleware]

	Middleware func(h http.Handler) http.Handler
)
