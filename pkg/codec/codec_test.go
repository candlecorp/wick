/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package codec_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/resolve"
)

func TestRegistry(t *testing.T) {
	r := codec.Registry{}

	loader := func(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
		return nil, nil
	}
	namedLoader := func() (string, bool, codec.Loader) {
		return "test", true, loader
	}

	r.Register(namedLoader)

	assert.Equal(t, fmt.Sprintf("%v", codec.Loader(loader)), fmt.Sprintf("%v", r["test"].Loader))
}
