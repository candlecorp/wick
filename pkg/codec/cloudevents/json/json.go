/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package json

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"

	"github.com/nanobus/nanobus/pkg/coalesce"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type (
	Config struct {
		SpecVersion string `mapstructure:"specversion"`
		Source      string `mapstructure:"source" structs:"-"`
	}

	// Codec encodes and decodes Avro records.
	Codec struct {
		config *Config
	}
)

// CloudEventsJSON is the NamedLoader for this codec.
func CloudEventsJSON() (string, bool, codec.Loader) {
	return "cloudevents+json", true, Loader
}

func Loader(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
	c := Config{
		SpecVersion: "1.0",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return NewCodec(&c), nil
}

// NewCodec creates a `Codec`.
func NewCodec(c *Config) *Codec {
	return &Codec{
		config: c,
	}
}

func (c *Codec) ContentType() string {
	return "application/cloudevents+json"
}

// Decode decodes JSON bytes to a value.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	var data map[string]interface{}
	if err := coalesce.JSONUnmarshal(msgValue, &data); err != nil {
		return nil, "", err
	}

	var typeValue string
	if typeField, ok := data["type"]; ok {
		typeValue, _ = typeField.(string)
	}

	return data, typeValue, nil
}

// Encode encodes a value into JSON encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	if m, ok := value.(map[string]interface{}); ok {
		if c.config.SpecVersion != "" {
			if _, exists := m["specversion"]; !exists {
				m["specversion"] = c.config.SpecVersion
			}
		}
		if _, exists := m["id"]; !exists {
			m["id"] = uuid.New().String()
		}
		if c.config.Source != "" {
			if _, exists := m["source"]; !exists {
				m["source"] = c.config.Source
			}
		}
		if _, exists := m["datacontenttype"]; !exists {
			data := m["data"]
			if _, ok := data.([]byte); ok {
				m["datacontenttype"] = "application/octet-stream"
			} else {
				m["datacontenttype"] = "application/json"
			}
		}
		if _, exists := m["time"]; !exists {
			m["time"] = time.Now().UTC().Format(time.RFC3339Nano)
		}
	}

	return json.Marshal(value)
}
