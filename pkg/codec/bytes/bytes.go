/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package bytes

import (
	"fmt"
	"reflect"

	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type Codec struct{}

// Load is the NamedLoader for this codec.
func Bytes() (string, bool, codec.Loader) {
	return "bytes", true, BytesLoader
}

func BytesLoader(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
	return New(), nil
}

func New() *Codec {
	return &Codec{}
}

func (c *Codec) ContentType() string {
	return "application/octet-stream"
}

// Decode decodes JSON bytes to a value.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	return msgValue, "", nil
}

// Encode encodes a value into JSON encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	switch val := value.(type) {
	case []byte:
		return val, nil
	case string:
		return []byte(val), nil
	}
	return nil, fmt.Errorf("incompatible binary type: %s", reflect.TypeOf(value).String())
}
