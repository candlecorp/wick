/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core_test

import (
	"bytes"
	"context"
	"errors"
	"io"
	"net/http"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/core"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/codec/json"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type DoFunc func(req *http.Request) (*http.Response, error)

type mockHTTPClient struct {
	req    *http.Request
	DoFunc DoFunc
}

func (m *mockHTTPClient) Do(req *http.Request) (*http.Response, error) {
	m.req = req
	resp, err := m.DoFunc(req)
	req.Response = resp
	return resp, err
}

func TestHTTP(t *testing.T) {
	ctx := context.Background()
	name, loader := core.HTTP()
	assert.Equal(t, "http", name)

	tests := []struct {
		name string

		do     DoFunc
		config map[string]interface{}

		data      actions.Data
		headers   http.Header
		output    interface{}
		loaderErr string
		actionErr string
	}{
		{
			name: "no body",
			do: func(req *http.Request) (*http.Response, error) {
				body := req.Body
				if body == nil {
					body = io.NopCloser(bytes.NewReader([]byte(``)))
				}

				return &http.Response{
					StatusCode: 200,
					Header:     req.Header,
					Body:       body,
				}, nil
			},
			config: map[string]interface{}{
				"url":    "https://test.io",
				"method": "GET",
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"name":        "test",
					"description": "full description",
					"nested": map[string]interface{}{
						"int":   1,
						"float": 1.1,
					},
				},
			},
			headers: http.Header{
				"Content-Type": []string{"application/json"},
			},
			output: nil,
		},
		{
			name: "input as body and output expr",
			do: func(req *http.Request) (*http.Response, error) {
				body := req.Body
				if body == nil {
					body = io.NopCloser(bytes.NewReader([]byte(``)))
				}

				return &http.Response{
					StatusCode: 200,
					Body:       body,
				}, nil
			},
			config: map[string]interface{}{
				"url":    "https://test.io",
				"method": "GET",
				"body":   "input",
				"headers": `{
	"X-Test": "test",
}`,
				"output": `{
	"name": name + "-TEST",
	"description": description,
	"nested": nested,
}`,
			},
			data: actions.Data{
				"input": map[string]interface{}{
					"name":        "test",
					"description": "full description",
					"nested": map[string]interface{}{
						"int":   1,
						"float": 1.1,
					},
				},
			},
			headers: http.Header{
				"Content-Type": []string{"application/json"},
				"X-Test":       []string{"test"},
			},
			output: map[string]interface{}{
				"name":        "test-TEST",
				"description": "full description",
				"nested": map[string]interface{}{
					"int":   int64(1),
					"float": float64(1.1),
				},
			},
		},
		{
			name: "http error",
			do: func(req *http.Request) (*http.Response, error) {
				return nil, errors.New("test error")
			},
			config: map[string]interface{}{
				"url":    "https://test.io",
				"method": "GET",
			},
			data: actions.Data{
				"input": map[string]interface{}{},
			},
			headers: http.Header{
				"Content-Type": []string{"application/json"},
			},
			actionErr: "test error",
		},
		{
			name: "http status non-200",
			do: func(req *http.Request) (*http.Response, error) {
				body := io.NopCloser(bytes.NewReader([]byte(`Not found`)))

				return &http.Response{
					StatusCode: 404,
					Body:       body,
				}, nil
			},
			config: map[string]interface{}{
				"url":    "https://test.io",
				"method": "GET",
			},
			data: actions.Data{
				"input": map[string]interface{}{},
			},
			headers: http.Header{
				"Content-Type": []string{"application/json"},
			},
			actionErr: "expected 2XX status code; received 404",
		},
		{
			name: "invalid config",
			config: map[string]interface{}{
				"url":       "https://test.io",
				"method":    "GET",
				"codecArgs": 1234,
			},
			data:      actions.Data{},
			loaderErr: "1 error(s) decoding:\n\n* 'codecArgs': source data must be an array or slice, got int",
		},
		{
			name: "unknown codec error",
			config: map[string]interface{}{
				"url":    "https://test.io",
				"method": "GET",
				"codec":  "unknown",
			},
			data:      actions.Data{},
			loaderErr: `unknown codec "unknown"`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := codec.Codecs{
				"json": json.NewCodec(),
			}
			client := mockHTTPClient{
				DoFunc: tt.do,
			}
			resolver := func(name string, target interface{}) bool {
				switch name {
				case "client:http":
					return resolve.As(&client, target)
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
			if assert.NotNil(t, client.req) && tt.headers != nil {
				assert.Equal(t, tt.headers, client.req.Header)
			}
		})
	}
}
