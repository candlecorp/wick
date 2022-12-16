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
	"errors"
	"testing"

	"github.com/nanobus/iota/go/payload"
	"github.com/nanobus/iota/go/rx/mono"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/core"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type mockInvoker struct {
	iface     string
	operation string
	input     payload.Payload
	output    payload.Payload
	err       error
}

func (m *mockInvoker) RequestResponse(ctx context.Context, iface, operation string, p payload.Payload) mono.Mono[payload.Payload] {
	m.iface = iface
	m.operation = operation
	m.input = p
	if m.err != nil {
		return mono.Error[payload.Payload](m.err)
	} else if m.output != nil {
		return mono.Just(m.output)
	} else if p != nil {
		return mono.Just(p)
	}
	panic("should return a payload or error")
}

func TestInvoke(t *testing.T) {
	ctx := context.Background()
	name, loader := core.Invoke()
	assert.Equal(t, "invoke", name)

	tests := []struct {
		name string

		invoker  *mockInvoker
		config   map[string]interface{}
		resolver resolve.ResolveAs

		data      actions.Data
		iface     string
		operation string
		output    interface{}
		loaderErr string
		actionErr string
	}{
		{
			name:    "normal input",
			invoker: &mockInvoker{},
			config: map[string]interface{}{
				"interface": "test.v1",
				"operation": "test",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"test": "test",
				},
			},
			output: map[string]interface{}{
				"test": "test",
			},
			iface:     "test.v1",
			operation: "test",
		},
		{
			name:    "normal input",
			invoker: &mockInvoker{},
			config: map[string]interface{}{
				"input": `{
					"test": input.test + "12345",
				}`,
				"interface": "test.v1",
				"operation": "test",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"test": "test",
				},
			},
			output: map[string]interface{}{
				"test": "test12345",
			},
			iface:     "test.v1",
			operation: "test",
		},
		{
			name:    "bytes input",
			invoker: &mockInvoker{},
			config: map[string]interface{}{
				"interface": "test.v1",
				"operation": "test",
			},
			data: actions.Data{
				"input": []byte(`{ "test": "test" }`),
			},
			output: map[string]interface{}{
				"test": "test",
			},
			iface:     "test.v1",
			operation: "test",
		},
		{
			name:    "string input",
			invoker: &mockInvoker{},
			config: map[string]interface{}{
				"interface": "test.v1",
				"operation": "test",
			},
			data: actions.Data{
				"input": `{ "test": "test" }`,
			},
			output: map[string]interface{}{
				"test": "test",
			},
			iface:     "test.v1",
			operation: "test",
		},
		{
			name:      "invoke from context",
			invoker:   &mockInvoker{},
			config:    map[string]interface{}{},
			data:      actions.Data{},
			iface:     "test.v1",
			operation: "test",
		},
		{
			name: "invoke error",
			invoker: &mockInvoker{
				err: errors.New("test error"),
			},
			config:    map[string]interface{}{},
			data:      actions.Data{},
			iface:     "test.v1",
			operation: "test",
			actionErr: "test error",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resolver := func(name string, target interface{}) bool {
				switch name {
				case "compute:mesh":
					return resolve.As(tt.invoker, target)
				}
				return false
			}

			action, err := loader(ctx, tt.config, resolver)
			if tt.loaderErr != "" {
				require.EqualError(t, err, tt.loaderErr, "loader error was expected")
				return
			}
			require.NoError(t, err, "loader failed")

			fctx := handler.ToContext(ctx, handler.Handler{
				Interface: tt.iface,
				Operation: tt.operation,
			})
			output, err := action(fctx, tt.data)
			if tt.actionErr != "" {
				require.EqualError(t, err, tt.actionErr, "action error was expected")
				return
			}
			require.NoError(t, err, "action failed")
			assert.Equal(t, tt.iface, tt.invoker.iface)
			assert.Equal(t, tt.operation, tt.invoker.operation)
			assert.Equal(t, tt.output, output)
		})
	}
}
