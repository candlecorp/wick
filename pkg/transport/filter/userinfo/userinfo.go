/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package userinfo

import (
	"context"
	"encoding/json"
	"io"
	"net/http"

	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/security/claims"
	"github.com/nanobus/nanobus/pkg/transport/filter"
)

type HTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}

type Config struct {
	UserInfoURL string `mapstructure:"userInfoUrl"`
	Debug       bool   `mapstructure:"debug"`
}

// UserInfo is the NamedLoader for the UserInfo filter.
func UserInfo() (string, filter.Loader) {
	return "userinfo", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (filter.Filter, error) {
	var c Config
	err := config.Decode(with, &c)
	if err != nil {
		return nil, err
	}

	var logger logr.Logger
	var httpClient HTTPClient
	if err := resolve.Resolve(resolver,
		"system:logger", &logger,
		"client:http", &httpClient); err != nil {
		return nil, err
	}

	return Filter(logger, httpClient, &c), nil
}

func Filter(log logr.Logger, httpClient HTTPClient, config *Config) filter.Filter {
	return func(ctx context.Context, header filter.Header) (context.Context, error) {
		if config.UserInfoURL == "" {
			return ctx, nil
		}

		authorization := header.Get("Authorization")
		if authorization == "" {
			return ctx, nil
		}

		req, err := http.NewRequest("GET", config.UserInfoURL, nil)
		if err != nil {
			return nil, err
		}
		req.Header.Add("Authorization", authorization)

		res, err := httpClient.Do(req)
		if err != nil {
			return nil, err
		}
		defer res.Body.Close()

		claimsJSON, err := io.ReadAll(res.Body)
		if err != nil {
			return nil, err
		}

		var c claims.Claims
		if err := json.Unmarshal(claimsJSON, &c); err != nil {
			return nil, err
		}

		ctx = claims.ToContext(ctx, c)

		if config.Debug {
			log.Info("Claims debug info [TURN OFF FOR PRODUCTION]",
				"claims", c)
		}

		return ctx, nil
	}
}
