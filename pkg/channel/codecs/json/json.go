/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package json

import (
	"encoding/json"
)

type Codec struct{}

func New() *Codec {
	return &Codec{}
}

func (c *Codec) ContentType() string {
	return "application/json"
}

func (c *Codec) Encode(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}

func (c *Codec) Decode(data []byte, v interface{}) error {
	return json.Unmarshal(data, v)
}
