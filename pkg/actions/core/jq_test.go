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

func TestJQ(t *testing.T) {
	ctx := context.Background()
	name, loader := core.JQ()
	assert.Equal(t, "jq", name)

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
				"query": `{cities: .locations | map(select(.state == "WA").name) | sort | join(", ") }`,
				"var":   `test`,
			},
			data: actions.Data{
				"locations": []interface{}{
					map[string]interface{}{"name": "Seattle", "state": "WA"},
					map[string]interface{}{"name": "New York", "state": "NY"},
					map[string]interface{}{"name": "Bellevue", "state": "WA"},
					map[string]interface{}{"name": "Olympia", "state": "WA"},
				},
			},
			output: []interface{}{
				map[string]interface{}{
					"cities": "Bellevue, Olympia, Seattle",
				},
			},
		},
		{
			name: "data input",
			config: map[string]interface{}{
				"query": `{cities: .locations | map(select(.state == "WA").name) | sort | join(", ") }`,
				"data":  "input",
				"var":   `test`,
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"locations": []interface{}{
						map[string]interface{}{"name": "Seattle", "state": "WA"},
						map[string]interface{}{"name": "New York", "state": "NY"},
						map[string]interface{}{"name": "Bellevue", "state": "WA"},
						map[string]interface{}{"name": "Olympia", "state": "WA"},
					},
				},
			},
			output: []interface{}{
				map[string]interface{}{
					"cities": "Bellevue, Olympia, Seattle",
				},
			},
		},
		{
			name: "single output",
			config: map[string]interface{}{
				"query":  `.locations | map(select(.state == "WA").name) | sort | first`,
				"single": true,
				"var":    `test`,
			},
			data: actions.Data{
				"locations": []interface{}{
					map[string]interface{}{"name": "Seattle", "state": "WA"},
					map[string]interface{}{"name": "New York", "state": "NY"},
					map[string]interface{}{"name": "Bellevue", "state": "WA"},
					map[string]interface{}{"name": "Olympia", "state": "WA"},
				},
			},
			output: "Bellevue",
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
