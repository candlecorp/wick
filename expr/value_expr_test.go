package expr_test

import (
	"testing"

	"github.com/nanobus/nanobus/expr"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
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
