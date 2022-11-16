/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package confluentavro

import (
	"encoding/json"
	"fmt"
	"io"
	"math/rand"
	"net/http"
	"net/url"
	"path"
	"sync/atomic"
)

// These numbers are used by the schema registry to communicate errors.
const (
	SubjectNotFound = 40401
	SchemaNotFound  = 40403
)

type (
	// A SchemaRegistry is a client for the confluent schema registry REST API (https://docs.confluent.io/current/schema-registry/docs/api.html).
	SchemaRegistry struct {
		currentURL uint32
		urls       []url.URL
		client     HTTPClient
	}

	// The RegisteredSchema type is an object produced by the schema registry.
	RegisteredSchema struct {
		Schema  string `json:"schema"`  // The actual AVRO schema
		Subject string `json:"subject"` // Subject where the schema is registered for
		Version int    `json:"version"` // Version within this subject
		ID      int    `json:"id"`      // Registry's unique id
	}

	// SimpleSchema encapsulates an Avro schema as a string.
	SimpleSchema struct {
		Schema string `json:"schema"`
	}

	// HTTPClient is the interface for making HTTP requests.
	HTTPClient interface {
		Do(req *http.Request) (resp *http.Response, err error)
	}
)

// A ConfluentError is an error as communicated by the schema registry.
// Some day this type might be exposed so that callers can do type assertions on it.
type ConfluentError struct {
	ErrorCode int    `json:"error_code"`
	Message   string `json:"message"`
}

// Parse handle parsing the RAW Avro schema returned from the schema registry into
// a `Schema` which is used by the `Encoder` and `Deccoder`.
func (s *RegisteredSchema) Parse() (*Schema, error) {
	return ParseSchema(s.ID, s.Schema)
}

// Error makes confluentError implement the error interface.
func (ce ConfluentError) Error() string {
	return fmt.Sprintf("%s (%d)", ce.Message, ce.ErrorCode)
}

// NewSchemaRegistry returns a new Client that connects to baseurl.
func NewSchemaRegistry(baseURLs []string, httpClient HTTPClient) (*SchemaRegistry, error) {
	urls := make([]url.URL, len(baseURLs))
	for i, baseURL := range baseURLs {
		u, err := url.Parse(baseURL)
		if err != nil {
			return nil, err
		}
		urls[i] = *u
	}
	currentURL := uint32(0)
	if len(urls) > 1 {
		currentURL = uint32(rand.Intn(len(urls) - 1))
	}
	return &SchemaRegistry{
		currentURL: currentURL,
		urls:       urls,
		client:     httpClient,
	}, nil
}

// Subjects returns all registered subjects.
func (c *SchemaRegistry) Subjects() (subjects []string, err error) {
	err = c.do("GET", "subjects", nil, &subjects)
	return
}

// Versions returns all schema version numbers registered for this subject.
func (c *SchemaRegistry) Versions(subject string) (versions []int, err error) {
	err = c.do("GET", fmt.Sprintf("subjects/%s/versions", subject), nil, &versions)
	return
}

// RegisterNewSchema registers the given schema for this subject.
func (c *SchemaRegistry) RegisterNewSchema(subject, schema string) (int, error) {
	var resp struct {
		ID int `json:"id"`
	}
	err := c.do("POST", fmt.Sprintf("/subjects/%s/versions", subject), SimpleSchema{schema}, &resp)
	return resp.ID, err
}

// IsRegistered tells if the given schema is registred for this subject.
func (c *SchemaRegistry) IsRegistered(subject, schema string) (bool, RegisteredSchema, error) {
	var fs RegisteredSchema
	err := c.do("POST", fmt.Sprintf("/subjects/%s", subject), SimpleSchema{schema}, &fs)
	// schema not found?
	if ce, confluentErr := err.(ConfluentError); confluentErr && ce.ErrorCode == SchemaNotFound {
		return false, fs, nil
	}
	// error?
	if err != nil {
		return false, fs, err
	}
	// so we have a schema then
	return true, fs, nil
}

// GetSchemaByID returns the schema for some id.
// The schema registry only provides the schema itself, not the id, subject or version.
func (c *SchemaRegistry) GetSchemaByID(id int) (string, error) {
	var s RegisteredSchema
	err := c.do("GET", fmt.Sprintf("/schemas/ids/%d", id), nil, &s)
	return s.Schema, err
}

// GetSchemaBySubject returns the schema for a particular subject and version.
func (c *SchemaRegistry) GetSchemaBySubject(subject string, ver int) (s RegisteredSchema, err error) {
	err = c.do("GET", fmt.Sprintf("/subjects/%s/versions/%d", subject, ver), nil, &s)
	return
}

// GetLatestSchema returns the latest version of the subject's schema.
func (c *SchemaRegistry) GetLatestSchema(subject string) (s RegisteredSchema, err error) {
	err = c.do("GET", fmt.Sprintf("/subjects/%s/versions/latest", subject), nil, &s)
	return
}

// do performs http requests and json (de)serialization.
func (c *SchemaRegistry) do(method, urlPath string, in interface{}, out interface{}) error {
	current := atomic.LoadUint32(&c.currentURL)
	tries := 0
	var resp *http.Response

	for {
		u := c.urls[current]
		u.Path = path.Join(u.Path, urlPath)
		var rdp io.Reader
		if in != nil {
			var wr *io.PipeWriter
			rdp, wr = io.Pipe()
			go func() {
				wr.CloseWithError(json.NewEncoder(wr).Encode(in))
			}()
		}
		req, err := http.NewRequest(method, u.String(), rdp)
		req.Header.Add("Accept", "application/vnd.schemaregistry.v1+json, application/vnd.schemaregistry+json, application/json")
		if err != nil {
			return err
		}
		resp, err = c.client.Do(req)
		if err != nil {
			if len(c.urls) > 1 {
				next := (current + 1) % uint32(len(c.urls))
				atomic.CompareAndSwapUint32(&c.currentURL, current, next)
			}

			tries++
			if tries >= len(c.urls) {
				return err // Number of tries are exhausted so return error.
			}
		} else {
			break
		}
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return parseSchemaRegistryError(resp)
	}
	return json.NewDecoder(resp.Body).Decode(out)
}

func parseSchemaRegistryError(resp *http.Response) error {
	var ce ConfluentError
	if err := json.NewDecoder(resp.Body).Decode(&ce); err != nil {
		return err
	}
	return ce
}
