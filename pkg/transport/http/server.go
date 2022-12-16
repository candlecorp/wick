/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package http

import (
	"context"
	"net"
	"net/http"

	"github.com/go-logr/logr"
	"github.com/gorilla/handlers"
	"github.com/gorilla/mux"
	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
	"go.opentelemetry.io/otel/trace"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/transport"
	"github.com/nanobus/nanobus/pkg/transport/http/middleware"
	"github.com/nanobus/nanobus/pkg/transport/http/router"
)

type Server struct {
	log     logr.Logger
	tracer  trace.Tracer
	address string
	handler http.Handler
	ln      net.Listener
}

type optionsHolder struct {
	middlewares []middleware.Middleware
	routers     []router.Router
}

type Option func(opts *optionsHolder)

func WithMiddleware(middlewares ...middleware.Middleware) Option {
	return func(opts *optionsHolder) {
		opts.middlewares = middlewares
	}
}

func WithRoutes(r ...router.Router) Option {
	return func(opts *optionsHolder) {
		opts.routers = r
	}
}

func HttpServerV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (transport.Transport, error) {
	var log logr.Logger
	var tracer trace.Tracer
	var routerRegistry router.Registry
	var middlewareRegistry middleware.Registry
	if err := resolve.Resolve(resolver,
		"system:logger", &log,
		"system:tracer", &tracer,
		"registry:routers", &routerRegistry,
		"registry:middleware", &middlewareRegistry); err != nil {
		return nil, err
	}

	// Defaults
	c := HttpServerV1Config{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	middlewares := make([]middleware.Middleware, len(c.Middleware))
	for i, component := range c.Middleware {
		m := middlewareRegistry[component.Uses]
		middleware, err := m(ctx, component.With, resolver)
		if err != nil {
			return nil, err
		}
		middlewares[i] = middleware
	}

	routers := make([]router.Router, len(c.Routes))
	for i, route := range c.Routes {
		r := routerRegistry[route.Uses]
		router, err := r(ctx, route.With, resolver)
		if err != nil {
			return nil, err
		}
		routers[i] = router
	}

	return NewServer(log, tracer, c,
		WithMiddleware(middlewares...),
		WithRoutes(routers...))
}

func NewServer(log logr.Logger, tracer trace.Tracer, config HttpServerV1Config, options ...Option) (*Server, error) {
	var opts optionsHolder

	for _, opt := range options {
		opt(&opts)
	}

	r := mux.NewRouter()
	r.Use(handlers.ProxyHeaders)

	for _, router := range opts.routers {
		if err := router(r, config.Address); err != nil {
			return nil, err
		}
	}

	var handler http.Handler = r
	if len(opts.middlewares) > 0 {
		for i := len(opts.middlewares) - 1; i >= 0; i-- {
			handler = opts.middlewares[i](handler)
		}
	}

	return &Server{
		log:     log,
		tracer:  tracer,
		address: config.Address,
		handler: handler,
	}, nil
}

func (t *Server) Listen() error {
	ln, err := net.Listen("tcp", t.address)
	if err != nil {
		return err
	}
	t.ln = ln
	t.log.Info("HTTP server listening", "address", t.address)

	handler := otelhttp.NewHandler(t.handler, "http")
	return http.Serve(ln, handler)
}

func (t *Server) Close() (err error) {
	if t.ln != nil {
		err = t.ln.Close()
		t.ln = nil
	}

	return err
}
