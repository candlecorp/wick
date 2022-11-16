/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package actions

import (
	"context"
	"errors"

	"github.com/nanobus/nanobus/pkg/registry"
)

type (
	NamedLoader = registry.NamedLoader[Action]
	Loader      = registry.Loader[Action]
	Registry    = registry.Registry[Action]

	Data   map[string]interface{}
	Action func(ctx context.Context, data Data) (interface{}, error)
)

func (d Data) Clone() Data {
	clone := make(Data, len(d)+5)
	for k, v := range d {
		clone[k] = v
	}
	return clone
}

// ErrStop is returned by an action when the processing should stop.
var ErrStop = errors.New("processing stopped")

func Stop() error {
	return ErrStop
}
