/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package text

import (
	"fmt"

	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type (
	// Codec encodes and decodes Avro records.
	Codec struct {
		contentType string
	}
)

// Plain is the NamedLoader for the `text/plain` codec.
func Plain() (string, bool, codec.Loader) {
	return "text/plain", true, func(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
		return NewCodec("text/plain"), nil
	}
}

// HTML is the NamedLoader for the `text/html` codec.
func HTML() (string, bool, codec.Loader) {
	return "text/html", true, func(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
		return NewCodec("text/html"), nil
	}
}

// NewCodec creates a `Codec`.
func NewCodec(contentType string) *Codec {
	return &Codec{
		contentType: contentType,
	}
}

func (c *Codec) ContentType() string {
	return c.contentType
}

// Decode decodes JSON bytes to a value.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	return string(msgValue), "", nil
}

// Encode encodes a value into JSON encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	msg := fmt.Sprintf("%v", value)
	return []byte(msg), nil
}
