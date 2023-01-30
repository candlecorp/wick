/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package resource

import (
	"fmt"
	"reflect"

	"github.com/nanobus/nanobus/pkg/registry"
)

type (
	Ref string

	NamedLoader = registry.NamedLoader[any]
	Loader      = registry.Loader[any]
	Registry    = registry.Registry[any]

	Resources map[string]interface{}
)

func Get[T any](r Resources, name Ref) (res T, err error) {
	var iface interface{}
	iface, ok := r[string(name)]
	if !ok {
		return res, fmt.Errorf("resource %q is not registered", name)
	}
	res, ok = iface.(T)
	if !ok {
		t := reflect.TypeOf(res)
		if t == nil {
			return res, fmt.Errorf("unknown target type trying to resolve resource %q", name)
		}
		return res, fmt.Errorf("resource %q is not a %s", name, t.Name())
	}

	return res, nil
}
