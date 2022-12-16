/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package router

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"reflect"
	"sort"

	"github.com/go-logr/logr"
	"github.com/gorilla/mux"

	"github.com/nanobus/nanobus/pkg/channel"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/transport"
	"github.com/nanobus/nanobus/pkg/transport/filter"
	"github.com/nanobus/nanobus/pkg/transport/http/router"
	"github.com/nanobus/nanobus/pkg/transport/httpresponse"
)

var (
	ErrUnregisteredContentType = errors.New("unregistered content type")
)

type Router struct {
	invoker       transport.Invoker
	errorResolver errorz.Resolver
	codecs        map[string]channel.Codec
	filters       []filter.Filter
}

func RouterV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (router.Router, error) {
	c := RouterV1Config{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var jsoncodec channel.Codec
	var msgpackcodec channel.Codec
	var transportInvoker transport.Invoker
	var logger logr.Logger
	if err := resolve.Resolve(resolver,
		"codec:json", &jsoncodec,
		"codec:msgpack", &msgpackcodec,
		"transport:invoker", &transportInvoker,
		"system:logger", &logger); err != nil {
		return nil, err
	}

	codecMap := channel.Codecs{
		jsoncodec.ContentType():    jsoncodec,
		msgpackcodec.ContentType(): msgpackcodec,
	}

	return NewV1(logger, transportInvoker, codecMap, c), nil
}

func NewV1(log logr.Logger, invoker transport.Invoker, codecMap channel.Codecs, config RouterV1Config) router.Router {
	router := Router{
		invoker: invoker,
	}
	return func(r *mux.Router, address string) error {
		sort.Slice(config, func(i, j int) bool {
			return len(config[i].URI) > len(config[j].URI)
		})
		for _, path := range config {
			path := path
			var desiredCodec channel.Codec
			if path.Encoding != nil {
				desiredCodec = codecMap[*path.Encoding]
			}
			log.Info("Serving route",
				"uri", path.URI,
				"methods", path.Methods,
				"handler", path.Handler)
			r.HandleFunc(path.URI, router.handler(path.Handler, desiredCodec)).
				Methods(path.Methods)
		}

		return nil
	}
}

func (t *Router) handler(h handler.Handler, desiredCodec channel.Codec) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		defer r.Body.Close()
		vars := mux.Vars(r)
		queryValues, _ := url.ParseQuery(r.URL.RawQuery)
		id := ""

		contentType := r.Header.Get("Content-Type")
		if contentType == "" {
			contentType = "application/json"
		}

		codec, ok := t.codecs[contentType]
		if !ok {
			w.WriteHeader(http.StatusUnsupportedMediaType)
			fmt.Fprintf(w, "%v: %s", ErrUnregisteredContentType, contentType)
			return
		}

		resp := httpresponse.New()
		ctx = httpresponse.NewContext(ctx, resp)

		for _, filter := range t.filters {
			var err error
			if ctx, err = filter(ctx, r.Header); err != nil {
				t.handleError(err, codec, r, w, http.StatusInternalServerError)
				return
			}
		}

		requestBytes, err := io.ReadAll(r.Body)
		if err != nil {
			t.handleError(err, codec, r, w, http.StatusInternalServerError)
			return
		}

		var body interface{}
		if len(requestBytes) > 0 {
			var body interface{}
			if err := codec.Decode(requestBytes, &body); err != nil {
				t.handleError(err, codec, r, w, http.StatusInternalServerError)
				return
			}
		}

		input := map[string]interface{}{
			"path":  vars,
			"query": queryValues,
			"body":  body,
		}

		if desiredCodec != nil {
			contentType := desiredCodec.ContentType()
			input["content_type"] = contentType
			if contentType == codec.ContentType() {
				input["data_bytes"] = requestBytes
			} else {
				targetBytes, err := desiredCodec.Encode(body)
				if err != nil {
					t.handleError(err, codec, r, w, http.StatusInternalServerError)
					return
				}
				input["data_bytes"] = targetBytes
			}
		}

		response, err := t.invoker(ctx, h.Interface, id, h.Operation, input)
		if err != nil {
			code := http.StatusInternalServerError
			if errors.Is(err, transport.ErrBadInput) {
				code = http.StatusBadRequest
			}
			t.handleError(err, codec, r, w, code)
			return
		}

		if !isNil(response) {
			header := w.Header()
			header.Set("Content-Type", codec.ContentType())
			for k, vals := range resp.Header {
				for _, v := range vals {
					header.Add(k, v)
				}
			}
			w.WriteHeader(resp.Status)
			responseBytes, err := codec.Encode(response)
			if err != nil {
				t.handleError(err, codec, r, w, http.StatusInternalServerError)
				return
			}

			w.Write(responseBytes)
		} else {
			w.WriteHeader(http.StatusNoContent)
		}
	}
}

func (t *Router) handleError(err error, codec channel.Codec, req *http.Request, w http.ResponseWriter, status int) {
	var errz *errorz.Error
	if !errors.As(err, &errz) {
		errz = t.errorResolver(err)
	}
	errz.Path = req.RequestURI

	w.Header().Add("Content-Type", codec.ContentType())
	w.WriteHeader(errz.Status)
	payload, err := codec.Encode(errz)
	if err != nil {
		fmt.Fprint(w, "unknown error")
	}

	w.Write(payload)
}

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
}
