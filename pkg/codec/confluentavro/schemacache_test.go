/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package confluentavro_test

import (
	"errors"
	"reflect"
	"testing"

	"github.com/nanobus/nanobus/pkg/codec/confluentavro"
)

type (
	mockSchemaRegistry struct {
		name    string
		counter int
	}
)

var (
	schemaString = `{
	"type": "record",
	"namespace": "com.example",
	"name": "FullName",
	"fields": [{
			"name": "first",
			"type": "string"
		},
		{
			"name": "last",
			"type": "string"
		}
	]
}`
	happySchema, _ = confluentavro.ParseSchema(0, schemaString)
)

func (m *mockSchemaRegistry) GetSchemaByID(id int) (string, error) {
	m.counter++
	switch m.name {
	case "schema-retrieval-error":
		return "", errors.New("woops")
	case "happy-path":
		return schemaString, nil
	case "schema-parse-error":
		return "bad", nil
	default:
		return "", nil
	}
}

func TestSchemaCache_GetSchema(t *testing.T) {
	// normal test cases
	subtests := []struct {
		name   string
		schema *confluentavro.Schema
		err    error
	}{
		{
			name:   "schema-retrieval-error",
			err:    errors.New("error getting schema ID 0: woops"),
			schema: nil,
		},
		{
			name:   "schema-parse-error",
			err:    errors.New("error parsing schema ID 0: avro: unknown type: bad"),
			schema: nil,
		},
		{
			name:   "happy-path",
			err:    nil,
			schema: happySchema,
		},
	}

	for _, tt := range subtests {
		t.Run(tt.name, func(t *testing.T) {
			mock := mockSchemaRegistry{name: tt.name}
			schemaCache := confluentavro.NewSchemaCache(&mock, 10)
			schema, err := schemaCache.GetSchema(0)
			if err != nil {
				if tt.err.Error() != err.Error() {
					t.Errorf("expected error (%v), got error (%v)", tt.err, err)
				}
			}
			if !reflect.DeepEqual(schema, tt.schema) {
				t.Errorf("expected schema (%v), got schema (%v)", tt.schema, schema)
			}
		})
	}

	// cache hit test cases
	mock := mockSchemaRegistry{name: "happy-path"}
	schemaCache := confluentavro.NewSchemaCache(&mock, 10)
	for x := 0; x < 5; x++ {
		schema, err := schemaCache.GetSchema(0) // counter should only increment the first time
		if err != nil {
			t.Errorf("got unexpected error from GetSchema: %v", schema)
		}
		if !reflect.DeepEqual(schema, happySchema) {
			t.Errorf("expected schema (%v), got schema (%v)", happySchema, schema)
		}
	}
	if mock.counter != 1 {
		t.Errorf("expected cache hits but retrieved schema %d times", mock.counter)
	}
}
