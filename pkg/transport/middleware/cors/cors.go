/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package cors

import (
	"context"
	"net/http"

	"github.com/rs/cors"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/transport/middleware"
)

type CorsConfig struct {
	// AllowedOrigins is a list of origins a cross-domain request can be executed from.
	// If the special "*" value is present in the list, all origins will be allowed.
	// An origin may contain a wildcard (*) to replace 0 or more characters
	// (i.e.: http://*.domain.com). Usage of wildcards implies a small performance penalty.
	// Only one wildcard can be used per origin.
	// Default value is ["*"]
	AllowedOrigins []string `mapstructure:"allowedOrigins"`
	// AllowedMethods is a list of methods the client is allowed to use with
	// cross-domain requests. Default value is simple methods (HEAD, GET and POST).
	AllowedMethods []string `mapstructure:"allowedMethods"`
	// AllowedHeaders is list of non simple headers the client is allowed to use with
	// cross-domain requests.
	// If the special "*" value is present in the list, all headers will be allowed.
	// Default value is [] but "Origin" is always appended to the list.
	AllowedHeaders []string `mapstructure:"allowedHeaders"`
	// ExposedHeaders indicates which headers are safe to expose to the API of a CORS
	// API specification
	ExposedHeaders []string `mapstructure:"exposedHeaders"`
	// MaxAge indicates how long (in seconds) the results of a preflight request
	// can be cached
	MaxAge int `mapstructure:"maxAge"`
	// AllowCredentials indicates whether the request can include user credentials like
	// cookies, HTTP authentication or client side SSL certificates.
	AllowCredentials bool `mapstructure:"allowCredentials"`
	// OptionsPassthrough instructs preflight to let other potential next handlers to
	// process the OPTIONS method. Turn this on if your application handles OPTIONS.
	OptionsPassthrough bool `mapstructure:"optionsPassthrough"`
	// Provides a status code to use for successful OPTIONS requests.
	// Default value is http.StatusNoContent (204).
	OptionsSuccessStatus int `mapstructure:"optionsSuccessStatus"`
	// Debugging flag adds additional output to debug server side CORS issues
	Debug bool `mapstructure:"debug"`

	// DevMode forces AllowedOrigins to *, AllowCredentials to true, and allows reflection
	// of the request Origin header. This works around a security protection embedded into
	// the standard that makes clients to refuse such configuration.
	// Obviously, this setting being set to true is only intended for development.
	DevMode bool `mapstructure:"devMode"`
}

// Cors is the NamedLoader for the cors middleware.
func Cors() (string, middleware.Loader) {
	return "cors/v0", CorsLoader
}

func CorsLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (middleware.Middleware, error) {
	c := CorsConfig{
		AllowedOrigins: []string{"*"},
		// "PUT", "PATCH", "DELETE" are commonly needed in REST APIs however
		// the defaults are aligned with the cors library defaults.
		AllowedMethods: []string{"HEAD", "GET", "POST"},
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	corsOptions := cors.Options{
		AllowedOrigins:       c.AllowedOrigins,
		AllowedMethods:       c.AllowedMethods,
		AllowedHeaders:       c.AllowedHeaders,
		ExposedHeaders:       c.ExposedHeaders,
		MaxAge:               c.MaxAge,
		AllowCredentials:     c.AllowCredentials,
		OptionsPassthrough:   c.OptionsPassthrough,
		OptionsSuccessStatus: c.OptionsSuccessStatus,
		Debug:                c.Debug,
	}

	if c.DevMode {
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

	return CorsHandler(corsOptions), nil
}

func CorsHandler(options cors.Options) middleware.Middleware {
	return func(h http.Handler) http.Handler {
		return cors.New(options).Handler(h)
	}
}
