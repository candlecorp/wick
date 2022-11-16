/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package structerror_test

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/structerror"
)

func TestCreate(t *testing.T) {
	tests := []struct {
		name     string
		create   func() *structerror.Error
		metadata map[string]string
		str      string
	}{
		{
			name: "new",
			create: func() *structerror.Error {
				return structerror.New("not_found",
					"key", "abcdef",
					"store", "statestore")
			},
			metadata: map[string]string{
				"key":   "abcdef",
				"store": "statestore",
			},
			str: `not_found
[key] abcdef
[store] statestore`,
		},
		{
			name: "parse",
			create: func() *structerror.Error {
				contents := `not_found
ignore
[key] abcdef
[store] statestore`
				return structerror.Parse(contents)
			},
			metadata: map[string]string{
				"key":   "abcdef",
				"store": "statestore",
			},
			str: `not_found
[key] abcdef
[store] statestore`,
		},
		{
			name: "parse no metadata",
			create: func() *structerror.Error {
				contents := `not_found`
				return structerror.Parse(contents)
			},
			metadata: nil,
			str:      `not_found`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := tt.create()
			assert.Equal(t, "not_found", e.Code())
			assert.Equal(t, tt.metadata, e.Metadata())

			assert.Equal(t, tt.str, e.Error())
			assert.Equal(t, tt.str, e.String())
		})
	}
}
