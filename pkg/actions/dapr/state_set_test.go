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

func TestStateSet(t *testing.T) {
	ctx := context.Background()

	type itemTest struct {
		key         string
		etag        string
		data        []byte
		consistency client.StateConsistency
		concurrency client.StateConcurrency
	}
	tests := []struct {
		name   string
		config map[string]any
		input  actions.Data
		items  []itemTest
	}{
		{
			name: "with key, etag, and options",
			config: map[string]any{
				"store": "test",
				"items": []any{
					map[string]any{
						"key":         `input.id`,
						"value":       "input.data",
						"etag":        `input.revision`,
						"consistency": "strong",
						"concurrency": "lastWrite",
					},
				},
			},
			input: actions.Data{
				"input": map[string]any{
					"id":       "1234",
					"revision": "5678",
					"data":     "Hello",
				},
			},
			items: []itemTest{
				{
					key:         "1234",
					etag:        "5678",
					data:        []byte(`"Hello"`),
					consistency: client.StateConsistencyStrong,
					concurrency: client.StateConcurrencyLastWrite,
				},
			},
		},
		{
			name: "foreach",
			config: map[string]any{
				"store": "test",
				"items": []any{
					map[string]any{
						"forEach":     `input.items`,
						"key":         `item.id`,
						"value":       "item.data",
						"etag":        `item.revision`,
						"consistency": "strong",
						"concurrency": "lastWrite",
					},
				},
			},
			input: actions.Data{
				"input": map[string]any{
					"items": []any{
						map[string]any{
							"id":       "1234",
							"revision": "5678",
							"data":     "Hello",
						},
					},
				},
			},
			items: []itemTest{
				{
					key:         "1234",
					etag:        "5678",
					data:        []byte(`"Hello"`),
					consistency: client.StateConsistencyStrong,
					concurrency: client.StateConcurrencyLastWrite,
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := mockClient{}
			resolver := getMockClient(&m)
			action, err := dapr.SetStateLoader(ctx, tt.config, resolver)
			require.NoError(t, err)
			_, err = action(ctx, tt.input)
			require.NoError(t, err)

			assert.Equal(t, "test", m.saveName)
			items := m.saveItems
			require.Len(t, items, len(tt.items))

			for i, item := range items {
				expected := tt.items[i]
				assert.Equal(t, expected.data, item.Value)
				if expected.etag != "" {
					require.NotNil(t, item.Etag)
					assert.Equal(t, expected.etag, item.Etag.Value)
				}
			}
		})
	}
}
