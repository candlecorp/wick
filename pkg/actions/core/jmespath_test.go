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
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/core"
)

func TestJMESPath(t *testing.T) {
	ctx := context.Background()
	name, loader := core.JMESPath()
	assert.Equal(t, "jmespath", name)

	tests := []struct {
		name      string
		config    map[string]interface{}
		data      actions.Data
		output    interface{}
		loaderErr string
		actionErr string
	}{
		{
			name: "normal input",
			config: map[string]interface{}{
				"path": `input.name`,
				"var":  `test`,
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"name":        "test",
					"description": "full description",
					"nested": map[string]interface{}{
						"int":   1,
						"float": 1.1,
					},
				},
			},
			output: "test",
		},
		{
			name: "data input",
			config: map[string]interface{}{
				"path": `nested.int`,
				"data": `input`,
				"var":  `test`,
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"name":        "test",
					"description": "full description",
					"nested": map[string]interface{}{
						"int":   1,
						"float": 1.1,
					},
				},
			},
			output: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			action, err := loader(ctx, tt.config, nil)
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
			assert.Equal(t, tt.output, output)
			if varName, ok := tt.config["var"]; ok {
				assert.Equal(t, tt.output, tt.data[fmt.Sprintf("%v", varName)])
			}
		})
	}
}
