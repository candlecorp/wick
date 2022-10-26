package function_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/function"
)

func TestContext(t *testing.T) {
	ctx := context.Background()
	empty := function.FromContext(ctx)
	assert.Equal(t, function.Function{}, empty)
	fn := function.Function{
		Namespace: "test.v1",
		Operation: "testing",
	}
	fctx := function.ToContext(ctx, fn)
	actual := function.FromContext(fctx)
	assert.Equal(t, fn, actual)
}
