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

	"github.com/nanobus/nanobus/pkg/channel/codecs/bytes"
)

func TestContentType(t *testing.T) {
	c := bytes.New()
	assert.Equal(t, "application/octet-stream", c.ContentType())
}

func TestBytes(t *testing.T) {
	value := []byte("test")
	c := bytes.New()

	var actual []byte
	c.Decode(value, &actual)
	assert.Equal(t, value, actual)

	out, err := c.Encode(actual)
	require.NoError(t, err)
	assert.Equal(t, actual, out)
}

func TestString(t *testing.T) {
	value := []byte("test")
	c := bytes.New()

	var actual string
	c.Decode(value, &actual)
	assert.Equal(t, "test", actual)

	out, err := c.Encode(actual)
	require.NoError(t, err)
	assert.Equal(t, value, out)
}
