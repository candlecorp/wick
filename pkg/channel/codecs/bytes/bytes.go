/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package bytes

import (
	"errors"
	"fmt"
	"reflect"
)

var (
	typeString = reflect.TypeOf("")       // nolint: gochecknoglobals
	typeBytes  = reflect.TypeOf([]byte{}) // nolint: gochecknoglobals
)

type Codec struct{}

func New() *Codec {
	return &Codec{}
}

func (c *Codec) ContentType() string {
	return "application/octet-stream"
}

func (c *Codec) Encode(v interface{}) ([]byte, error) {
	switch val := v.(type) {
	case []byte:
		return val, nil
	case string:
		return []byte(val), nil
	}
	return nil, fmt.Errorf("incompatible binary type: %s", reflect.TypeOf(v).String())
}

func (c *Codec) Decode(data []byte, v interface{}) error {
	val := reflect.ValueOf(v)
	typ := val.Type()
	if typ.Kind() != reflect.Ptr || val.IsNil() {
		return errors.New("expecting a non-nil pointer")
	}

	typ = typ.Elem()
	val = val.Elem()

	switch typ {
	case typeString:
		val.SetString(string(data))
		return nil
	case typeBytes:
		val.SetBytes(data)
		return nil
	}

	return fmt.Errorf("unexpected type for binary: %s", typ.String())
}
