/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package msgpack

import (
	"github.com/vmihailenco/msgpack/v5"

	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type (
	// Codec encodes and decodes Avro records.
	Codec struct{}
)

// MsgPack is the NamedLoader for this codec.
func MsgPack() (string, bool, codec.Loader) {
	return "msgpack", true, Loader
}

func Loader(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
	return NewCodec(), nil
}

// NewCodec creates a `Codec`.
func NewCodec() *Codec {
	return &Codec{}
}

func (c *Codec) ContentType() string {
	return "application/msgpack"
}

// Decode decodes MsgPack bytes to a value.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	var data interface{}
	if err := msgpack.Unmarshal(msgValue, &data); err != nil {
		return nil, "", err
	}

	return data, "", nil
}

// Encode encodes a value into MsgPack encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	return msgpack.Marshal(value)
}
