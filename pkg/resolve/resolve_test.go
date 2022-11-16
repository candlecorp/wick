/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package resolve_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/resolve"
)

func TestResolve(t *testing.T) {
	dep1 := "test 1"
	dep2 := "test 2"
	deps := func(name string) (interface{}, bool) {
		switch name {
		case "1":
			return dep1, true
		case "2":
			return dep2, true
		}
		return nil, false
	}
	resolveAs := resolve.ToResolveAs(deps)

	var target1 string
	var target2 string

	err := resolve.Resolve(resolveAs,
		"1")
	assert.EqualError(t, err, "invalid number of arguments passed to Resolve")

	err = resolve.Resolve(resolveAs,
		1, &target1)
	assert.EqualError(t, err, "argument 0 is not a string")

	err = resolve.Resolve(resolveAs,
		"1", "2")
	assert.EqualError(t, err, "could not resolve dependency \"1\"")

	err = resolve.Resolve(resolveAs,
		"unknown", &target1)
	assert.EqualError(t, err, "could not resolve dependency \"unknown\"")

	err = resolve.Resolve(resolveAs,
		"1", nil)
	assert.EqualError(t, err, "could not resolve dependency \"1\"")

	var incorrectTarget int
	err = resolve.Resolve(resolveAs,
		"1", &incorrectTarget)
	assert.EqualError(t, err, "could not resolve dependency \"1\"")

	err = resolve.Resolve(resolveAs,
		"1", &target1,
		"2", &target2)
	require.NoError(t, err)
	assert.Equal(t, dep1, target1)
	assert.Equal(t, dep2, target2)
}
