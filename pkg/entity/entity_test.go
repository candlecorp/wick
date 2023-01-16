/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package entity_test

import (
	"testing"

	"github.com/nanobus/nanobus/pkg/entity"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestContext(t *testing.T) {
	expected := entity.Entity{
		Namespace: "test.v1",
		Type:      "Blog",
	}
	str := expected.String()
	var actual entity.Entity
	err := actual.FromString(str)
	require.NoError(t, err)
	assert.Equal(t, expected, actual)
}
