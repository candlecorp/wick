package core_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/actions/core"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/security/claims"
)

func TestAuthorize(t *testing.T) {
	ctx := context.Background()
	name, loader := core.Authorize()
	assert.Equal(t, "authorize", name)

	tests := []struct {
		name string

		claims claims.Claims
		config map[string]interface{}

		resolver  resolve.ResolveAs
		data      actions.Data
		expected  interface{}
		loaderErr string
		actionErr string
	}{
		{
			name: "using has",
			claims: claims.Claims{
				"test": true,
			},
			config: map[string]interface{}{
				"has": []string{"test"},
			},
			data: actions.Data{},
		},
		{
			name: "using has - unauthorized",
			claims: claims.Claims{
				"nottest": true,
			},
			config: map[string]interface{}{
				"has": []string{"test"},
			},
			data:      actions.Data{},
			actionErr: "permission_denied\n[claim] test",
		},
		{
			name: "using check",
			claims: claims.Claims{
				"test": true,
			},
			config: map[string]interface{}{
				"check": map[string]interface{}{
					"test": true,
				},
			},
			data: actions.Data{},
		},
		{
			name: "using check - permission_denied",
			claims: claims.Claims{
				"test": true,
			},
			config: map[string]interface{}{
				"check": map[string]interface{}{
					"test": false,
				},
			},
			data:      actions.Data{},
			actionErr: "permission_denied\n[claim] test\n[want] false",
		},
		{
			name: "using condition",
			claims: claims.Claims{
				"test": true,
			},
			config: map[string]interface{}{
				"condition": "claims.test == true",
			},
			data: actions.Data{
				"claims": claims.Claims{
					"test": true,
				},
			},
		},
		{
			name: "using condition - unauthorized",
			claims: claims.Claims{
				"test": true,
			},
			config: map[string]interface{}{
				"condition": "claims.test == false",
			},
			data: actions.Data{
				"claims": claims.Claims{
					"test": true,
				},
			},
			actionErr: "permission_denied\n[expr] claims.test == false",
		},
		{
			name: "unauthorized message",
			claims: claims.Claims{
				"nottest": true,
			},
			config: map[string]interface{}{
				"has":   []string{"test"},
				"error": "whomp whomp",
			},
			data:      actions.Data{},
			actionErr: "whomp whomp\n[claim] test",
		},
		{
			name: "loader error",
			config: map[string]interface{}{
				"has": ` notaslice`,
			},
			loaderErr: "1 error(s) decoding:\n\n* 'has': source data must be an array or slice, got string",
		},
		{
			name: "expression error",
			claims: claims.Claims{
				"test": true,
			},
			config: map[string]interface{}{
				"condition": "claims.test == true",
			},
			data:      actions.Data{},
			actionErr: "cannot fetch test from <nil> (1:8)\n | claims.test == true\n | .......^",
		},
		{
			name: "non-boolean expression",
			claims: claims.Claims{
				"test": true,
			},
			config: map[string]interface{}{
				"condition": "12345",
			},
			data: actions.Data{
				"claims": claims.Claims{
					"test": true,
				},
			},
			actionErr: "expression \"12345\" did not evaluate a boolean",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			action, err := loader(ctx, tt.config, tt.resolver)
			if tt.loaderErr != "" {
				require.EqualError(t, err, tt.loaderErr, "loader error was expected")
				return
			}

			require.NoError(t, err, "loader failed")

			cctx := claims.ToContext(ctx, tt.claims)
			output, err := action(cctx, tt.data)
			if tt.actionErr != "" {
				require.EqualError(t, err, tt.actionErr, "action error was expected")
				return
			}
			require.NoError(t, err, "action failed")
			assert.Equal(t, tt.expected, output)
		})
	}
}
