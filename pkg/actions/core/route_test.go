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
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/core"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/runtime"
)

type runner func(ctx context.Context, data actions.Data) (interface{}, error)

type mockProcessor struct {
	pipeline  *runtime.Pipeline
	runnables map[string]runner
	ran       []string
	err       error
}

var _ = (core.Processor)((*mockProcessor)(nil))

func (m *mockProcessor) LoadPipeline(pl *runtime.Pipeline) (runtime.Runnable, error) {
	m.pipeline = pl
	fn := m.runnables[pl.Name]

	runnable := mockRunnable{m, pl.Name, fn}

	return runnable.Run, m.err
}

func (m *mockProcessor) Interface(ctx context.Context, h handler.Handler, data actions.Data) (interface{}, bool, error) {
	return data, true, nil
}

func (m *mockProcessor) Provider(ctx context.Context, h handler.Handler, data actions.Data) (interface{}, bool, error) {
	return data, true, nil
}

type mockRunnable struct {
	m       *mockProcessor
	summary string
	fn      runner
}

func (m mockRunnable) Run(ctx context.Context, data actions.Data) (interface{}, error) {
	m.m.ran = append(m.m.ran, m.summary)
	return m.fn(ctx, data)
}

func TestRoute(t *testing.T) {
	ctx := context.Background()
	name, loader := core.Route()
	assert.Equal(t, "route", name)

	tests := []struct {
		name string

		config    map[string]interface{}
		processor *mockProcessor

		data      actions.Data
		pipeline  runtime.Pipeline
		expected  interface{}
		ran       []string
		loaderErr string
		actionErr string
	}{
		{
			name: "single",
			config: map[string]interface{}{
				"selection": "single",
				"routes": []interface{}{
					map[string]interface{}{
						"name": "A",
						"when": `path == 'A'`,
						"then": []interface{}{
							map[string]interface{}{
								"name": "1",
								"uses": "test a",
							},
						},
					},
					map[string]interface{}{
						"name": "B",
						"when": `path == 'B'`,
						"then": []interface{}{
							map[string]interface{}{
								"name": "1",
								"uses": "test b",
							},
						},
					},
				},
			},
			processor: &mockProcessor{
				runnables: map[string]runner{
					"B": func(ctx context.Context, data actions.Data) (interface{}, error) {
						return "b", nil
					},
				},
			},
			pipeline: runtime.Pipeline{
				Name: "B",
				Steps: []runtime.Step{
					{
						Name: "1",
						Uses: "test b",
					},
				},
			},
			expected: "b",
			ran:      []string{"B"},
			data: actions.Data{
				"path": "B",
			},
		},
		{
			name: "multi",
			config: map[string]interface{}{
				"selection": "multi",
				"routes": []interface{}{
					map[string]interface{}{
						"name": "A",
						"when": `path == 'A'`,
						"then": []interface{}{
							map[string]interface{}{
								"name": "1",
								"uses": "test a",
							},
						},
					},
					map[string]interface{}{
						"name": "B",
						"when": `other == 'B'`,
						"then": []interface{}{
							map[string]interface{}{
								"name": "1",
								"uses": "test b",
							},
						},
					},
				},
			},
			processor: &mockProcessor{
				runnables: map[string]runner{
					"A": func(ctx context.Context, data actions.Data) (interface{}, error) {
						return "a", nil
					},
					"B": func(ctx context.Context, data actions.Data) (interface{}, error) {
						return "b", nil
					},
				},
			},
			pipeline: runtime.Pipeline{
				Name: "B",
				Steps: []runtime.Step{
					{
						Name: "1",
						Uses: "test b",
					},
				},
			},
			expected: "b",
			ran:      []string{"A", "B"},
			data: actions.Data{
				"path":  "A",
				"other": "B",
			},
		},
		{
			name: "configuration error",
			config: map[string]interface{}{
				"selection": "invalid",
				"routes":    1234,
			},
			data:      actions.Data{},
			loaderErr: "2 error(s) decoding:\n\n* 'routes': source data must be an array or slice, got int\n* error decoding 'selection': invalid SelectionMode \"invalid\": unknown value \"invalid\" for SelectionMode",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resolver := func(name string, target interface{}) bool {
				switch name {
				case "system:processor":
					return resolve.As(tt.processor, target)
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
			assert.Equal(t, tt.ran, tt.processor.ran)
			assert.Equal(t, &tt.pipeline, tt.processor.pipeline)
			assert.Equal(t, tt.expected, output)
		})
	}
}
