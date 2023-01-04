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

func TestStateDelete(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name        string
		config      map[string]any
		input       actions.Data
		inputKey    string
		inputEtag   string
		consistency client.StateConsistency
		concurrency client.StateConcurrency
	}{
		{
			name: "with key",
			config: map[string]any{
				"store": "test",
				"key":   `input.id`,
			},
			input: actions.Data{
				"input": map[string]any{
					"id": "1234",
				},
			},
			inputKey: "1234",
		},
		{
			name: "with key, etag, and options",
			config: map[string]any{
				"store":       "test",
				"key":         `input.id`,
				"etag":        `input.revision`,
				"consistency": "strong",
				"concurrency": "lastWrite",
			},
			input: actions.Data{
				"input": map[string]any{
					"id":       "1234",
					"revision": "5678",
				},
			},
			inputKey:    "1234",
			inputEtag:   "5678",
			consistency: client.StateConsistencyStrong,
			concurrency: client.StateConcurrencyLastWrite,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := mockClient{}
			resolver := getMockClient(&m)
			action, err := dapr.DeleteStateLoader(ctx, tt.config, resolver)
			require.NoError(t, err)
			_, err = action(ctx, tt.input)
			require.NoError(t, err)

			assert.Equal(t, "test", m.deleteName)
			assert.Equal(t, tt.inputKey, m.deleteKey)
			if tt.inputEtag != "" {
				require.NotNil(t, m.deleteEtag)
				assert.Equal(t, tt.inputEtag, m.deleteEtag.Value)
			}
			if m.deleteOpts != nil {
				assert.Equal(t, tt.consistency, m.deleteOpts.Consistency)
				assert.Equal(t, tt.concurrency, m.deleteOpts.Concurrency)
			}
		})
	}
}
