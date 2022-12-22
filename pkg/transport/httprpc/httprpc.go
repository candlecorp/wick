/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package httprpc

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"

	"github.com/go-logr/logr"
	"github.com/gorilla/mux"

	"github.com/nanobus/nanobus/pkg/channel"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/spec"
	"github.com/nanobus/nanobus/pkg/transport"
	"github.com/nanobus/nanobus/pkg/transport/filter"
)

type HTTPRPC struct {
	log           logr.Logger
	address       string
	invoker       transport.Invoker
	errorResolver errorz.Resolver
	codecs        map[string]channel.Codec
	filters       []filter.Filter
	ln            net.Listener
}

type optionsHolder struct {
	codecs  []channel.Codec
	filters []filter.Filter
}

var (
	ErrUnregisteredContentType = errors.New("unregistered content type")
	ErrInvalidURISyntax        = errors.New("invalid invocation URI syntax")
)

type Option func(opts *optionsHolder)

func WithCodecs(codecs ...channel.Codec) Option {
	return func(opts *optionsHolder) {
		opts.codecs = codecs
	}
}

func WithFilters(filters ...filter.Filter) Option {
	return func(opts *optionsHolder) {
		opts.filters = filters
	}
}

type Configuration struct {
	Address string `mapstructure:"address" validate:"required"`
}

func Load() (string, transport.Loader) {
	return "httprpc", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (transport.Transport, error) {
	var jsoncodec channel.Codec
	var msgpackcodec channel.Codec
	var transportInvoker transport.Invoker
	var namespaces spec.Namespaces
	var errorResolver errorz.Resolver
	var filters []filter.Filter
	var log logr.Logger
	if err := resolve.Resolve(resolver,
		"codec:json", &jsoncodec,
		"codec:msgpack", &msgpackcodec,
		"transport:invoker", &transportInvoker,
		"spec:namespaces", &namespaces,
		"errors:resolver", &errorResolver,
		"filter:lookup", &filters,
		"system:logger", &log); err != nil {
		return nil, err
	}

	var c Configuration
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return New(log, c.Address, namespaces, transportInvoker, errorResolver,
		WithFilters(filters...),
		WithCodecs(jsoncodec, msgpackcodec))
}

func New(log logr.Logger, address string, namespaces spec.Namespaces, invoker transport.Invoker, errorResolver errorz.Resolver, options ...Option) (transport.Transport, error) {
	var opts optionsHolder

	for _, opt := range options {
		opt(&opts)
	}

	codecMap := make(map[string]channel.Codec, len(opts.codecs))
	for _, c := range opts.codecs {
		codecMap[c.ContentType()] = c
	}

	return &HTTPRPC{
		log:           log,
		address:       address,
		invoker:       invoker,
		errorResolver: errorResolver,
		codecs:        codecMap,
		filters:       opts.filters,
	}, nil
}

func (t *HTTPRPC) Listen() error {
	r := mux.NewRouter()
	r.HandleFunc("/{interface}/{operation}", t.handler).Methods("POST")
	r.HandleFunc("/{interface}/{id}/{operation}", t.handler).Methods("POST")
	r.Use(mux.CORSMethodMiddleware(r))
	ln, err := net.Listen("tcp", t.address)
	if err != nil {
		return err
	}
	t.ln = ln
	t.log.Info("HTTP RPC server listening", "address", t.address)

	return http.Serve(ln, r)
}

func (t *HTTPRPC) Close() (err error) {
	if t.ln != nil {
		err = t.ln.Close()
		t.ln = nil
	}

	return err
}

func (t *HTTPRPC) handler(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()

	contentType := r.Header.Get("Content-Type")
	if contentType == "" {
		contentType = "application/json"
	}

	ctx := r.Context()

	codec, ok := t.codecs[contentType]
	if !ok {
		w.WriteHeader(http.StatusUnsupportedMediaType)
		fmt.Fprintf(w, "%v: %s", ErrUnregisteredContentType, contentType)
		return
	}

	operation := mux.Vars(r)["operation"]
	id := mux.Vars(r)["id"]

	var h handler.Handler
	if err := h.FromString(operation); err != nil {
		t.handleError(err, codec, r, w, http.StatusInternalServerError)
		return
	}

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

	var input interface{}
	if len(requestBytes) > 0 {
		if err := codec.Decode(requestBytes, &input); err != nil {
			t.handleError(err, codec, r, w, http.StatusInternalServerError)
			return
		}
	} else {
		input = map[string]interface{}{}
	}

	response, err := t.invoker(ctx, h, id, input, transport.PerformAuthorization)
	if err != nil {
		code := http.StatusInternalServerError
		if errors.Is(err, transport.ErrBadInput) {
			code = http.StatusBadRequest
		}
		t.handleError(err, codec, r, w, code)
		return
	}

	w.Header().Set("Content-Type", codec.ContentType())
	responseBytes, err := codec.Encode(response)
	if err != nil {
		t.handleError(err, codec, r, w, http.StatusInternalServerError)
		return
	}

	w.Write(responseBytes)
}

func (t *HTTPRPC) handleError(err error, codec channel.Codec, req *http.Request, w http.ResponseWriter, status int) {
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
