/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package blob

import (
	"github.com/nanobus/nanobus/pkg/actions"
)

var All = []actions.NamedLoader{
	Read,
	Write,
}
