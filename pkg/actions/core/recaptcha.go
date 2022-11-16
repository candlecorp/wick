/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"time"

	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type ReCaptchaConfig struct {
	SiteVerifyURL string          `mapstructure:"siteVerifyUrl"`
	Secret        string          `mapstructure:"secret" validate:"required"`
	Response      *expr.ValueExpr `mapstructure:"response" validate:"required"`
	Score         float64         `mapstructure:"score"`
	Action        string          `mapstructure:"action"`
}

type SiteVerifyResponse struct {
	Success     bool      `json:"success"`
	Score       float64   `json:"score"`
	Action      string    `json:"action"`
	ChallengeTS time.Time `json:"challenge_ts"`
	Hostname    string    `json:"hostname"`
	ErrorCodes  []string  `json:"error-codes"`
}

// ReCaptcha is the NamedLoader for the reCaptcha action.
func ReCaptcha() (string, actions.Loader) {
	return "recaptcha", ReCaptchaLoader
}

func ReCaptchaLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := ReCaptchaConfig{
		SiteVerifyURL: "https://www.google.com/recaptcha/api/siteverify",
		Score:         0.5,
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var logger logr.Logger
	var httpClient HTTPClient
	if err := resolve.Resolve(resolver,
		"system:logger", &logger,
		"client:http", &httpClient); err != nil {
		return nil, err
	}

	return ReCaptchaAction(logger, httpClient, &c), nil
}

func ReCaptchaAction(
	log logr.Logger,
	httpClient HTTPClient,
	config *ReCaptchaConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		// Get challenge response
		responseIface, err := config.Response.Eval(data)
		if err != nil {
			return nil, err
		}
		responseVal := fmt.Sprintf("%v", responseIface)

		req, err := http.NewRequest(http.MethodPost, config.SiteVerifyURL, nil)
		if err != nil {
			return nil, err
		}

		// Add necessary request parameters.
		q := req.URL.Query()
		q.Add("secret", config.Secret)
		q.Add("response", responseVal)
		req.URL.RawQuery = q.Encode()

		// Make request
		resp, err := httpClient.Do(req)
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()

		// Decode response.
		var response SiteVerifyResponse
		if err = json.NewDecoder(resp.Body).Decode(&response); err != nil {
			return nil, err
		}

		log.Info("reCaptcha verify", "response", response)

		// Check recaptcha verification success.
		if !response.Success {
			return nil, errors.New("unsuccessful recaptcha verify request")
		}

		// Check response score.
		if response.Score < config.Score {
			return nil, errors.New("lower received score than expected")
		}

		// Check response action.
		if config.Action != "" && response.Action != config.Action {
			return nil, errors.New("mismatched recaptcha action")
		}

		return nil, nil
	}
}
