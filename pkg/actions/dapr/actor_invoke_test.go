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

func TestInvokeActor(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name      string
		config    map[string]any
		data      actions.Data
		actorType string
		actorID   string
		method    string
		input     any
		output    any
		err       error
	}{
		{
			name: "default input",
			config: map[string]any{
				"handler": "Test::test",
				"id":      "input.id",
			},
			data: actions.Data{
				"input": map[string]any{
					"id": "1234",
				},
			},
			actorType: "Test",
			actorID:   "1234",
			method:    "test",
			input: map[string]any{
				"id": "1234",
			},
			output: "test",
		},
		{
			name: "with input and metadata",
			config: map[string]any{
				"handler": "Test::test",
				"id":      "input.id",
				"data":    `input.content`,
			},
			data: actions.Data{
				"input": map[string]any{
					"id": "1234",
					"content": map[string]any{
						"foo": "bar",
					},
				},
			},
			actorType: "Test",
			actorID:   "1234",
			method:    "test",
			input: map[string]any{
				"foo": "bar",
			},
			output: "test",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var resp *daprc.InvokeActorResponse
			if tt.output != nil {
				outBytes, _ := json.Marshal(tt.output)
				resp = &daprc.InvokeActorResponse{
					Data: outBytes,
				}
			}

			m := mockClient{
				actorResp: resp,
				actorErr:  tt.err,
			}
			resolver := getMockClient(&m)
			action, err := dapr.InvokeActorLoader(ctx, tt.config, resolver)
			require.NoError(t, err)
			output, err := action(ctx, tt.data)
			require.NoError(t, err)

			require.NotNil(t, m.actorReq)
			assert.Equal(t, tt.actorType, m.actorReq.ActorType)
			assert.Equal(t, tt.actorID, m.actorReq.ActorID)
			assert.Equal(t, tt.method, m.actorReq.Method)

			var actual any
			require.NoError(t, json.Unmarshal(m.actorReq.Data, &actual))
			assert.Equal(t, tt.input, actual)

			if tt.output != nil {
				assert.Equal(t, tt.output, output)
			}
		})
	}
}
