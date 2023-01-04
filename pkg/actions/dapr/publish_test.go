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

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/dapr"
)

func TestPublish(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name      string
		config    map[string]any
		input     actions.Data
		inputData []byte
	}{
		{
			name: "with input and metadata",
			config: map[string]any{
				"pubsub":   "test",
				"topic":    "test",
				"data":     `input.data`,
				"key":      `input.key`,
				"metadata": `meta`,
			},
			input: actions.Data{
				"input": map[string]any{
					"key": "12345",
					"data": map[string]any{
						"test": "test",
					},
				},
				"meta": map[string]any{
					"test": "test",
				},
			},
			inputData: []byte(`{"test":"test"}`),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := mockClient{}
			resolver := getMockClient(&m)
			action, err := dapr.PublishLoader(ctx, tt.config, resolver)
			require.NoError(t, err)
			_, err = action(ctx, tt.input)
			require.NoError(t, err)

			assert.Equal(t, tt.inputData, m.publishData)
		})
	}
}
