/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package confluentavro_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/codec/confluentavro"
)

type cache struct {
	schema *confluentavro.Schema
}

func (c *cache) GetSchema(id int) (*confluentavro.Schema, error) {
	return c.schema, nil
}

var schema *confluentavro.Schema

func init() {
	schemaJSON := `{
		"type": "record",
		"name": "ExampleRecord",
		"namespace": "com.acme.messages",
		"fields": [
			{
				"name": "someProperty",
				"type": [
					"null",
					"string"
				]
			},
			{
				"name": "otherProperty",
				"type": {
					"type": "record",
					"name": "NestedRecord",
					"fields": [
						{
							"name": "nestedProperty",
							"type": "string"
						}
					]
				}
			}
		]
	}`
	var err error
	schema, err = confluentavro.ParseSchema(1, schemaJSON)
	if err != nil {
		panic(err)
	}
}

func TestEncodeDecode(t *testing.T) {
	record := map[string]interface{}{
		"someProperty": "foo",
		"otherProperty": map[string]interface{}{
			"nestedProperty": "bar",
		},
	}

	codec := confluentavro.NewCodec(&cache{schema: schema})
	encodedBytes, err := codec.Encode(record, schema.ID())
	require.Nil(t, err)
	require.Equal(t, []byte{0, 0, 0, 0, 1, 2, 6, 102, 111, 111, 6, 98, 97, 114}, encodedBytes)

	readI, _, err := codec.Decode(encodedBytes)
	require.NoError(t, err)
	read, ok := readI.(map[string]interface{})
	require.True(t, ok)

	assert.Equal(t, "foo", read["someProperty"])
	otherProperty, ok := read["otherProperty"].(map[string]interface{})
	require.True(t, ok)
	assert.Equal(t, "bar", otherProperty["nestedProperty"])
}
