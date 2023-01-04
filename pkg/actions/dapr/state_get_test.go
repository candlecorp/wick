/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package dapr_test

import (
	"context"
	"testing"

	"github.com/dapr/go-sdk/client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/dapr"
)

func TestStateGet(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name        string
		config      map[string]any
		input       actions.Data
		inputKey    string
		consistency client.StateConsistency
		concurrency client.StateConcurrency
		stateItem   *client.StateItem
		output      any
		err         string
	}{
		{
			name: "with key and options",
			config: map[string]any{
				"store":       "test",
				"key":         `input.id`,
				"consistency": "strong",
				"concurrency": "lastWrite",
			},
			input: actions.Data{
				"input": map[string]any{
					"id": "1234",
				},
			},
			inputKey:    "1234",
			consistency: client.StateConsistencyStrong,
			concurrency: client.StateConcurrencyLastWrite,
			stateItem: &client.StateItem{
				Value: []byte(`"Hello"`),
			},
			output: `Hello`,
		},
		{
			name: "not found",
			config: map[string]any{
				"store":         "test",
				"key":           `input.id`,
				"consistency":   "strong",
				"concurrency":   "lastWrite",
				"notFoundError": "test_not_found",
			},
			input: actions.Data{
				"input": map[string]any{
					"id": "1234",
				},
			},
			inputKey:    "1234",
			consistency: client.StateConsistencyStrong,
			concurrency: client.StateConcurrencyLastWrite,
			stateItem:   &client.StateItem{}, // No value
			err:         "test_not_found\n[key] 1234\n[resource] dapr",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := mockClient{
				getItem: tt.stateItem,
			}
			resolver := getMockClient(&m)
			action, err := dapr.GetStateLoader(ctx, tt.config, resolver)
			require.NoError(t, err)
			output, err := action(ctx, tt.input)

			assert.Equal(t, "test", m.getName)
			if tt.err != "" {
				assert.EqualError(t, err, tt.err)
			} else {
				require.NoError(t, err)
				assert.Equal(t, tt.inputKey, m.getKey)
				assert.Equal(t, tt.output, output)
			}
		})
	}
}
