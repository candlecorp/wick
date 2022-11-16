/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package routes

import (
	"github.com/gorilla/mux"
	"github.com/nanobus/nanobus/pkg/registry"
)

type (
	NamedLoader = registry.NamedLoader[AddRoutes]
	Loader      = registry.Loader[AddRoutes]
	Registry    = registry.Registry[AddRoutes]

	AddRoutes func(r *mux.Router)
)
