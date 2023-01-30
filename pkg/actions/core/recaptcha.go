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
	"net/http"
	"time"

	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resolve"
)

type SiteVerifyResponse struct {
	Success     bool      `json:"success" yaml:"success"`
	Score       float64   `json:"score" yaml:"score"`
	Action      string    `json:"action" yaml:"action"`
	ChallengeTs time.Time `json:"challengeTs" yaml:"challengeTs"`
	Hostname    string    `json:"hostname" yaml:"hostname"`
	ErrorCodes  []string  `json:"errorCodes" yaml:"errorCodes"`
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
		responseVal, err := expr.EvalAsStringE(config.Response, data)
		if err != nil {
			return nil, err
		}

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
			return nil, resiliency.Retriable(err)
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
		if config.Action != nil && response.Action != *config.Action {
			return nil, errors.New("mismatched recaptcha action")
		}

		return nil, nil
	}
}
