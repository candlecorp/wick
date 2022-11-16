/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package confluentavro

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"

	"github.com/hamba/avro"
	"github.com/spf13/cast"

	"github.com/nanobus/nanobus/pkg/coalesce"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type Config struct {
	SchemaRegistryURLs []string `mapstructure:"schemaRegistryURLs"`
	SchemaCacheSize    int      `mapstructure:"schemaCacheSize"`
}

// ConfluentAvro is the NamedLoader for this codec.
func ConfluentAvro() (string, bool, codec.Loader) {
	return "confluentavro", false, Loader
}

func Loader(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
	c := Config{
		SchemaCacheSize: 200,
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var httpClient HTTPClient
	if err := resolve.Resolve(resolver,
		"client:http", &httpClient); err != nil {
		return nil, err
	}

	registry, err := NewSchemaRegistry(c.SchemaRegistryURLs, httpClient)
	if err != nil {
		return nil, err
	}

	cache := NewSchemaCache(registry, c.SchemaCacheSize)

	return NewCodec(cache), nil
}

type (
	// Codec encodes and decodes Avro records.
	Codec struct {
		schemaRetriever schemaRetriever
	}

	// Dependencies

	schemaRetriever interface {
		GetSchema(id int) (*Schema, error)
	}
)

// Errors
var (
	// ErrBadMessage denotes that the message is not a valid Avro message with a Schema ID.
	ErrBadMessage = errors.New("bad message")
	// ErrUnknownMagicByte denotes that an invalid magic byte was encountered.
	ErrUnknownMagicByte = errors.New("unknown magic byte")
	// ErrValidationFailed denotes that the datum was invalid per the schema.
	ErrValidationFailed = errors.New("schema validation failed")
	// ErrEnvelopeMissing denotes that the datum is missing the enterprise envelope.
	ErrEnvelopeMissing = errors.New("datum is missing an enterprise envelope")
)

// NewCodec creates a `Codec`.
func NewCodec(schemaCache schemaRetriever) *Codec {
	return &Codec{
		schemaRetriever: schemaCache,
	}
}

func (c *Codec) ContentType() string {
	return "application/avro"
}

// Decode decodes the record into a map using the schema id and raw avro bytes from the message.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	if len(msgValue) < 5 {
		return nil, "", ErrBadMessage
	}
	if msgValue[0] != 0 {
		return nil, "", ErrUnknownMagicByte
	}

	schemaID := int(binary.BigEndian.Uint32(msgValue[1:]))
	schema, err := c.schemaRetriever.GetSchema(schemaID)
	if err != nil {
		return nil, "", fmt.Errorf("could not retrieve schema ID %d: %w", schemaID, err)
	}

	reader := bytes.NewReader(msgValue[5:])
	decoder := avro.NewDecoderForSchema(schema.schema, reader)

	var v map[string]interface{}
	if err = decoder.Decode(&v); err != nil {
		return nil, "", fmt.Errorf("could not retrieve schema ID %d: %w", schemaID, err)
	}

	return v, schema.typeName, nil
}

// Encode validates and encodes a map into Avro encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	if len(args) < 1 {
		return nil, errors.New("confluentavro: encode: missing arguments")
	}

	schemaID, err := cast.ToIntE(args[0])
	if err != nil {
		return nil, fmt.Errorf("invalid valid value %v for schemaID: %w", args[0], err)
	}
	schema, err := c.schemaRetriever.GetSchema(schemaID)
	if err != nil {
		return nil, fmt.Errorf("could not retrieve schema ID %d: %w", schemaID, err)
	}

	buf := bytes.Buffer{}
	// Write magic byte
	if err = buf.WriteByte(0); err != nil {
		return nil, fmt.Errorf("failed to write magic byte: %w", err)
	}
	// Write schema ID
	if err = binary.Write(&buf, binary.BigEndian, uint32(schema.ID())); err != nil {
		return nil, fmt.Errorf("failed to write schema ID: %w", err)
	}

	decoder := avro.NewEncoderForSchema(schema.schema, &buf)

	coalesce.Unsigned(value)
	if err = decoder.Encode(value); err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}
