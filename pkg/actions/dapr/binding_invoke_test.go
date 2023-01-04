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
	"encoding/json"
	"testing"

	daprc "github.com/dapr/go-sdk/client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/dapr"
)

func TestInvokeBinding(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name          string
		config        map[string]any
		input         actions.Data
		inputData     []byte
		inputMetadata []byte
		output        any
	}{
		{
			name: "no input",
			config: map[string]any{
				"binding":   "test",
				"operation": "test",
			},
			input: actions.Data{
				"input": "test",
			},
		},
		{
			name: "with input and metadata",
			config: map[string]any{
				"binding":   "test",
				"operation": "test",
				"data":      `input`,
				"metadata":  `meta`,
			},
			input: actions.Data{
				"input": "test",
				"meta": map[string]any{
					"test": "test",
				},
			},
			inputData:     []byte(`"test"`),
			inputMetadata: []byte(`{"test": "test"}`),
			output:        "test",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var eventOut *daprc.BindingEvent
			if tt.output != nil {
				outBytes, _ := json.Marshal(tt.output)
				eventOut = &daprc.BindingEvent{
					Data: outBytes,
				}
			}

			m := mockClient{
				bindingOut: eventOut,
			}
			resolver := getMockClient(&m)
			action, err := dapr.InvokeBindingLoader(ctx, tt.config, resolver)
			require.NoError(t, err)
			output, err := action(ctx, tt.input)
			require.NoError(t, err)

			if tt.inputData != nil {
				assert.Equal(t, tt.inputData, m.bindingReq.Data)
			}

			if tt.output != nil {
				assert.Equal(t, tt.output, output)
			}
		})
	}
}
