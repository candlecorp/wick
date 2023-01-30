/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/core"
	"github.com/nanobus/nanobus/pkg/resolve"
)

func TestExpr(t *testing.T) {
	ctx := context.Background()
	name, loader := core.Expr()
	assert.Equal(t, "expr", name)

	tests := []struct {
		name string

		config   map[string]interface{}
		resolver resolve.ResolveAs

		data      actions.Data
		expected  interface{}
		loaderErr string
		actionErr string
	}{
		{
			name: "using value expression",
			config: map[string]interface{}{
				"value": "input",
				"to":    "test",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"test": "1234",
				},
			},
			expected: map[string]interface{}{
				"test": "1234",
			},
		},
		{
			name: "using data expression",
			config: map[string]interface{}{
				"data": `{ "test": input.test }`,
				"to":   "test",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"test": "1234",
				},
			},
			expected: map[string]interface{}{
				"test": "1234",
			},
		},
		{
			name: "loader error",
			config: map[string]interface{}{
				"value": `^@&#$RFSDF`,
				"to":    "test",
			},
			loaderErr: "1 error(s) decoding:\n\n* error decoding 'value': invalid ValueExpr \"^@&#$RFSDF\": unrecognized character: U+0040 '@' (1:3)\n | ^@&#$RFSDF\n | ..^",
		},
		{
			name: "value error",
			config: map[string]interface{}{
				"value": "fail_notfound",
				"to":    "test",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"test": "1234",
				},
			},
			expected: nil,
		},
		{
			name: "data error",
			config: map[string]interface{}{
				"data": "notfound",
				"to":   "test",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"test": "1234",
				},
			},
			actionErr: "execute error: undefined symbol 'notfound'",
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

			output, err := action(ctx, tt.data)
			if tt.actionErr != "" {
				require.EqualError(t, err, tt.actionErr, "action error was expected")
				return
			}
			require.NoError(t, err, "action failed")
			assert.Equal(t, tt.expected, output)
			if to, ok := tt.config["to"]; ok {
				assert.Equal(t, output, tt.data[to.(string)])
			}
		})
	}
}
