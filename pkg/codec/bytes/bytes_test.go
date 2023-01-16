/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package bytes_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/codec/bytes"
)

func TestContentType(t *testing.T) {
	c := bytes.New()
	assert.Equal(t, "application/octet-stream", c.ContentType())
}

func TestBytes(t *testing.T) {
	value := []byte("test")
	c := bytes.New()

	actual, _, err := c.Decode(value)
	require.NoError(t, err)
	assert.Equal(t, value, actual)

	out, err := c.Encode(actual)
	require.NoError(t, err)
	assert.Equal(t, actual, out)
}

func TestString(t *testing.T) {
	c := bytes.New()
	out, err := c.Encode("test")
	require.NoError(t, err)
	assert.Equal(t, []byte("test"), out)
}
