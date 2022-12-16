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
	"strings"

	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/security/claims"
	"github.com/nanobus/nanobus/pkg/transport/filter"
)

type HTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}

func UserInfoV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (filter.Filter, error) {
	var c UserInfoV1Config
	err := config.Decode(with, &c)
	if err != nil {
		return nil, err
	}

	var logger logr.Logger
	var httpClient HTTPClient
	var developerMode bool
	if err := resolve.Resolve(resolver,
		"system:logger", &logger,
		"client:http", &httpClient,
		"developerMode", &developerMode); err != nil {
		return nil, err
	}

	return Filter(logger, httpClient, &c, developerMode), nil
}

func Filter(log logr.Logger, httpClient HTTPClient, config *UserInfoV1Config, developerMode bool) filter.Filter {
	return func(ctx context.Context, header filter.Header) (context.Context, error) {
		if config.UserInfoURL == "" {
			return ctx, nil
		}

		authorization := header.Get("Authorization")
		if authorization == "" {
			return ctx, nil
		}

		// Ignore JWTs
		if strings.HasPrefix(authorization, "Bearer ") {
			tokenString := authorization[7:]
			// Check for the prefix of all JWTs.
			if strings.HasPrefix(tokenString, "ey") {
				return ctx, nil
			}
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

		if developerMode {
			log.Info("Claims debug info [TURN OFF FOR PRODUCTION]",
				"component", "userinfo",
				"claims", c)
		}

		return ctx, nil
	}
}
