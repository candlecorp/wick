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

func TestFilter(t *testing.T) {
	ctx := context.Background()
	name, loader := core.Filter()
	assert.Equal(t, "filter", name)

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
			name: "continue",
			config: map[string]interface{}{
				"condition": "test == true",
			},
			data: actions.Data{
				"test": true,
			},
		},
		{
			name: "stop",
			config: map[string]interface{}{
				"condition": "test == false",
			},
			data: actions.Data{
				"test": true,
			},
			actionErr: actions.ErrStop.Error(),
		},
		{
			name: "non-boolean expression",
			config: map[string]interface{}{
				"condition": "12345",
			},
			data: actions.Data{
				"test": true,
			},
			actionErr: "expression \"12345\" did not evaluate a boolean",
		},
		{
			name: "loader error",
			config: map[string]interface{}{
				"condition": 12345,
			},
			loaderErr: "1 error(s) decoding:\n\n* 'condition' expected a map, got 'int'",
		},
		{
			name: "expression error",
			config: map[string]interface{}{
				"condition": "test.test == true",
			},
			data:      actions.Data{},
			actionErr: "cannot fetch test from <nil> (1:6)\n | test.test == true\n | .....^",
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
		})
	}
}
