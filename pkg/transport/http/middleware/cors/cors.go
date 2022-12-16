/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package cors

import (
	"context"
	"net/http"

	"github.com/rs/cors"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/transport/http/middleware"
)

func CorsV0Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (middleware.Middleware, error) {
	c := CorsV0Config{
		AllowedOrigins: []string{"*"},
		// "PUT", "PATCH", "DELETE" are commonly needed in REST APIs however
		// the defaults are aligned with the cors library defaults.
		AllowedMethods:       []string{"HEAD", "GET", "POST"},
		OptionsSuccessStatus: 204,
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var developerMode bool
	if err := resolve.Resolve(resolver,
		"developerMode", &developerMode); err != nil {
		return nil, err
	}

	maxAge := int(0)
	if c.MaxAge != nil {
		maxAge = int(*c.MaxAge)
	}

	corsOptions := cors.Options{
		AllowedOrigins:       c.AllowedOrigins,
		AllowedMethods:       c.AllowedMethods,
		AllowedHeaders:       c.AllowedHeaders,
		ExposedHeaders:       c.ExposedHeaders,
		MaxAge:               maxAge,
		AllowCredentials:     c.AllowCredentials,
		OptionsPassthrough:   c.OptionsPassthrough,
		OptionsSuccessStatus: int(c.OptionsSuccessStatus),
		Debug:                developerMode,
	}

	// Developer mode forces AllowedOrigins to *, AllowCredentials to true, and allows reflection
	// of the request Origin header. This works around a security protection embedded into
	// the standard that makes clients to refuse such configuration.
	// Obviously, this setting being set to true is only intended for development.
	if developerMode {
		corsOptions.AllowedOrigins = []string{}
		corsOptions.AllowCredentials = true

		// Documentation from github.com/rs/cors
		//
		// ### Allow * With Credentials Security Protection
		//
		// This library has been modified to avoid a well known security
		//  issue when configured with `AllowedOrigins` to `*` and
		// `AllowCredentials` to `true`. Such setup used to make the library
		// reflects the request `Origin` header value, working around a
		// security protection embedded into the standard that makes clients
		// to refuse such configuration. This behavior has been removed with
		// [#55](https://github.com/rs/cors/issues/55) and
		// [#57](https://github.com/rs/cors/issues/57).
		//
		// If you depend on this behavior and understand the implications, you
		// can restore it using the
		// `AllowOriginFunc` with `func(origin string) {return true}`.
		//
		// Please refer to [#55](https://github.com/rs/cors/issues/55) for
		// more information about the security implications.
		corsOptions.AllowOriginFunc = func(origin string) bool { return true }
	}

	return CorsV0Handler(corsOptions), nil
}

func CorsV0Handler(options cors.Options) middleware.Middleware {
	return func(h http.Handler) http.Handler {
		return cors.New(options).Handler(h)
	}
}
