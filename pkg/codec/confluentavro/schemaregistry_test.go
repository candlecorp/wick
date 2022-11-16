/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package confluentavro_test

import (
	"bytes"
	"encoding/json"
	"io/ioutil"
	"net/http"
	"reflect"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/codec/confluentavro"
)

const testHost = "testhost:1337"
const testUrl = "http://" + testHost

type D func(req *http.Request) (*http.Response, error)

func (d D) Do(req *http.Request) (*http.Response, error) {
	return d(req)
}

// verifies the http.Request, creates an http.Response
func dummyHttpHandler(t *testing.T, method, path string, status int, reqBody, respBody interface{}) D {
	d := D(func(req *http.Request) (*http.Response, error) {
		if method != "" && req.Method != method {
			t.Errorf("method is wrong, expected `%s`, got `%s`", method, req.Method)
		}
		if req.URL.Host != testHost {
			t.Errorf("expected host `%s`, got `%s`", testHost, req.URL.Host)
		}
		if path != "" && req.URL.Path != path {
			t.Errorf("path is wrong, expected `%s`, got `%s`", path, req.URL.Path)
		}
		if reqBody != nil {
			expbs, err := json.Marshal(reqBody)
			require.NoError(t, err)
			bs, err := ioutil.ReadAll(req.Body)
			require.NoError(t, err)
			mustEqual(t, strings.Trim(string(bs), "\r\n"), strings.Trim(string(expbs), "\r\n"))
		}
		var resp http.Response
		resp.StatusCode = status
		if respBody != nil {
			bs, err := json.Marshal(respBody)
			require.NoError(t, err)
			resp.Body = ioutil.NopCloser(bytes.NewReader(bs))
		}
		return &resp, nil
	})
	return d
}

func httpSuccess(t *testing.T, method, path string, reqBody, respBody interface{}) *confluentavro.SchemaRegistry {
	client, _ := confluentavro.NewSchemaRegistry([]string{testUrl}, dummyHttpHandler(t, method, path, 200, reqBody, respBody))
	return client
}

func httpError(t *testing.T, status, errCode int, errMsg string) *confluentavro.SchemaRegistry {
	client, _ := confluentavro.NewSchemaRegistry([]string{testUrl}, dummyHttpHandler(t, "", "", status, nil, confluentavro.ConfluentError{
		ErrorCode: errCode,
		Message:   errMsg,
	}))
	return client
}

func mustEqual(t *testing.T, actual, expected interface{}) {
	if !reflect.DeepEqual(actual, expected) {
		t.Errorf("expected `%#v`, got `%#v`", expected, actual)
	}
}

func TestSubjects(t *testing.T) {
	subsIn := []string{"rollulus", "hello-subject"}
	c := httpSuccess(t, "GET", "/subjects", nil, subsIn)
	subs, err := c.Subjects()
	if err != nil {
		t.Error()
	}
	mustEqual(t, subs, subsIn)
}

func TestVersions(t *testing.T) {
	versIn := []int{1, 2, 3}
	c := httpSuccess(t, "GET", "/subjects/mysubject/versions", nil, versIn)
	vers, err := c.Versions("mysubject")
	if err != nil {
		t.Error()
	}
	mustEqual(t, vers, versIn)
}

func TestIsRegistered_yes(t *testing.T) {
	s := `{"x":"y"}`
	ss := confluentavro.SimpleSchema{Schema: s}
	sIn := confluentavro.RegisteredSchema{Schema: s, Subject: "mysubject", Version: 4, ID: 7}
	c := httpSuccess(t, "POST", "/subjects/mysubject", ss, sIn)
	isreg, sOut, err := c.IsRegistered("mysubject", s)
	if err != nil {
		t.Error()
	}
	if !isreg {
		t.Error()
	}
	mustEqual(t, sOut, sIn)
}

func TestIsRegistered_not(t *testing.T) {
	c := httpError(t, 404, confluentavro.SchemaNotFound, "too bad")
	isreg, _, err := c.IsRegistered("mysubject", "{}")
	if err != nil {
		t.Error()
	}
	if isreg {
		t.Error()
	}
}
