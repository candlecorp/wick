/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package actions_test

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/actions"
)

func TestClone(t *testing.T) {
	data := actions.Data{
		"one": 1234,
	}
	clone := data.Clone()
	assert.Equal(t, data, clone)
	assert.NotSame(t, data, clone)
}

func TestStop(t *testing.T) {
	assert.Equal(t, actions.ErrStop, actions.Stop())
}
