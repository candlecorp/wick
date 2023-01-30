/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package entity

import (
	"encoding/json"
	"fmt"
	"strings"
)

type Entity struct {
	Namespace string `json:"namespace" msgpack:"namespace"`
	Type      string `json:"type" msgpack:"type"`
}

func (e *Entity) String() string {
	return e.Namespace + "::" + e.Type
}

func (e *Entity) FromString(typeName string) error {
	parts := strings.Split(typeName, "::")
	if len(parts) != 2 {
		return fmt.Errorf("invalid entity format %q", typeName)
	}
	*e = Entity{
		Namespace: parts[0],
		Type:      parts[1],
	}
	return nil
}

// UnmarshalJSON unmashals a quoted json string to the enum value
func (e *Entity) UnmarshalJSON(b []byte) error {
	var str string
	err := json.Unmarshal(b, &str)
	if err != nil {
		return err
	}
	return e.FromString(str)
}

func (h *Entity) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}
	return h.FromString(str)
}
