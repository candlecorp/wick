/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package router

import (
	"github.com/gorilla/mux"

	"github.com/nanobus/nanobus/pkg/registry"
)

type (
	NamedLoader = registry.NamedLoader[Router]
	Loader      = registry.Loader[Router]
	Registry    = registry.Registry[Router]

	Router func(r *mux.Router, address string) error
)
