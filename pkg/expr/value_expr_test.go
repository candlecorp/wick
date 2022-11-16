/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package expr_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/expr"
)

func TestValueExpr(t *testing.T) {
	var ve expr.ValueExpr
	err := ve.DecodeString(`input.test != nil && result.test == 5678`)
	require.NoError(t, err)
	result, err := ve.Eval(map[string]interface{}{
		"input": map[string]interface{}{
			"test": 1234,
		},
		"result": map[string]interface{}{
			"test": 5678,
		},
	})
	require.NoError(t, err)
	assert.Equal(t, true, result)
}

var result interface{}

func BenchmarkEval(b *testing.B) {
	var ve expr.ValueExpr
	err := ve.DecodeString(`input.test != nil && result.test == 5678`)
	require.NoError(b, err)
	data := map[string]interface{}{
		"input": map[string]interface{}{
			"test": 1234,
		},
		"result": map[string]interface{}{
			"test": 5678,
		},
	}
	var r interface{}
	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		r, _ = ve.Eval(data)
	}
	result = r
}
