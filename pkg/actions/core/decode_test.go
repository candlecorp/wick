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
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/core"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type mockDecoder struct {
	codec.Codec
}

// Decode decodes JSON bytes to a value.
func (m *mockDecoder) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	var data interface{}
	if err := json.Unmarshal(msgValue, &data); err != nil {
		return nil, "", err
	}

	return data, "json", nil
}

func TestDecode(t *testing.T) {
	ctx := context.Background()
	name, loader := core.Decode()
	assert.Equal(t, "decode", name)

	tests := []struct {
		name string

		config map[string]interface{}

		data      actions.Data
		output    interface{}
		loaderErr string
		actionErr string
	}{
		{
			name: "bytes",
			config: map[string]interface{}{
				"dataField": "input.bytes",
				"typeField": "decodedType",
				"codec":     "json",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"bytes": []byte(`{"name": "test"}`),
				},
			},
			output: map[string]interface{}{
				"name": "test",
			},
		},
		{
			name: "string",
			config: map[string]interface{}{
				"dataField": "input.string",
				"typeField": "decodedType",
				"codec":     "json",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"string": `{"name": "test"}`,
				},
			},
			output: map[string]interface{}{
				"name": "test",
			},
		},
		{
			name: "invalid config",
			config: map[string]interface{}{
				"codecArgs": 1234,
			},
			data:      actions.Data{},
			loaderErr: "1 error(s) decoding:\n\n* 'codecArgs': source data must be an array or slice, got int",
		},
		{
			name: "unknown codec error",
			config: map[string]interface{}{
				"dataField": "input.bytes",
				"codec":     "unknown",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"bytes": `{"name": "test"}`,
				},
			},
			loaderErr: `unknown codec "unknown"`,
		},
		{
			name: "non-map error",
			config: map[string]interface{}{
				"dataField": "input.invalid",
				"codec":     "json",
			},
			data: actions.Data{
				"input": 1234,
			},
			actionErr: `non-map encountered for property "invalid"`,
		},
		{
			name: "property not set",
			config: map[string]interface{}{
				"dataField": "input.invalid",
				"codec":     "json",
			},
			data:      actions.Data{},
			actionErr: `property "input" not set`,
		},
		{
			name: "invalid data value",
			config: map[string]interface{}{
				"dataField": "input.invalid",
				"codec":     "json",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"invalid": 1234,
				},
			},
			actionErr: `"input.invalid" must be []byte or string for decoding`,
		},
		{
			name: "decoding error",
			config: map[string]interface{}{
				"dataField": "input.invalid",
				"codec":     "json",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"invalid": `{"name": "test"`,
				},
			},
			actionErr: `unexpected end of JSON input`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := codec.Codecs{
				"json": &mockDecoder{},
			}
			resolver := func(name string, target interface{}) bool {
				switch name {
				case "codec:lookup":
					return resolve.As(c, target)
				}
				return false
			}

			action, err := loader(ctx, tt.config, resolver)
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
			if typeField, ok := tt.config["typeField"]; ok {
				assert.Equal(t, "json", tt.data[typeField.(string)])
			}
		})
	}
}
