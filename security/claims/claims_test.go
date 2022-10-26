package claims_test

import (
	"context"
	"testing"

	"github.com/nanobus/nanobus/security/claims"
	"github.com/stretchr/testify/assert"
)

func TestCmobine(t *testing.T) {
	cl1 := claims.Claims{
		"name": "test",
	}
	cl2 := claims.Claims{
		"override": 1234,
	}
	cl3 := claims.Claims{
		"override": 5678,
	}
	cl4 := claims.Claims{
		"role": "admin",
	}
	cl := claims.Combine(cl1, cl2, cl3, cl4, nil)
	assert.Equal(t, claims.Claims{
		"name":     "test",
		"override": 5678,
		"role":     "admin",
	}, cl)
}

func TestContext(t *testing.T) {
	ctx := context.Background()
	empty := claims.FromContext(ctx)
	assert.Equal(t, claims.Claims{}, empty)
	cl := claims.Claims{
		"role": "admin",
	}
	fctx := claims.ToContext(ctx, nil)
	actual := claims.FromContext(fctx)
	assert.Equal(t, claims.Claims{}, actual)
	fctx = claims.ToContext(ctx, cl)
	actual = claims.FromContext(fctx)
	assert.Equal(t, cl, actual)
}
