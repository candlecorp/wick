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

	"github.com/nanobus/nanobus/pkg/coalesce"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type (
	// Codec encodes and decodes Avro records.
	Codec struct{}
)

// JSON is the NamedLoader for this codec.
func JSON() (string, bool, codec.Loader) {
	return "json", true, Loader
}

func Loader(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
	return NewCodec(), nil
}

// NewCodec creates a `Codec`.
func NewCodec() *Codec {
	return &Codec{}
}

func (c *Codec) ContentType() string {
	return "application/json"
}

// Decode decodes JSON bytes to a value.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	var data interface{}
	if err := coalesce.JSONUnmarshal(msgValue, &data); err != nil {
		return nil, "", err
	}

	return data, "", nil
}

// Encode encodes a value into JSON encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	return json.Marshal(value)
}
