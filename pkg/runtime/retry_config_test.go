/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package runtime

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/resiliency/retry"
)

func TestRetryDecode(t *testing.T) {
	tests := map[string]struct {
		config    map[string]interface{}
		overrides func(config *retry.Config)
		err       string
	}{
		"invalid policy type": {
			config: map[string]interface{}{
				"policy": "invalid",
			},
			overrides: nil,
			err:       "1 error(s) decoding:\n\n* error decoding 'policy': invalid PolicyType \"invalid\": unexpected back off policy type: invalid",
		},
		"default": {
			config:    map[string]interface{}{},
			overrides: nil,
			err:       "",
		},
		"constant default": {
			config: map[string]interface{}{
				"policy": "constant",
			},
			overrides: nil,
			err:       "",
		},
		"constant with duration": {
			config: map[string]interface{}{
				"policy":   "constant",
				"duration": "10s",
			},
			overrides: func(config *retry.Config) {
				config.Duration = 10 * time.Second
			},
			err: "",
		},
		"exponential default": {
			config: map[string]interface{}{
				"policy": "exponential",
			},
			overrides: func(config *retry.Config) {
				config.Policy = retry.PolicyExponential
			},
			err: "",
		},
		"exponential with string settings": {
			config: map[string]interface{}{
				"policy":              "exponential",
				"initialInterval":     "1000", // 1s
				"randomizationFactor": "1.0",
				"multiplier":          "2.0",
				"maxInterval":         "120000",  // 2m
				"maxElapsedTime":      "1800000", // 30m
			},
			overrides: func(config *retry.Config) {
				config.Policy = retry.PolicyExponential
				config.InitialInterval = 1 * time.Second
				config.RandomizationFactor = 1.0
				config.Multiplier = 2.0
				config.MaxInterval = 2 * time.Minute
				config.MaxElapsedTime = 30 * time.Minute
			},
			err: "",
		},
		"exponential with typed settings": {
			config: map[string]interface{}{
				"policy":              "exponential",
				"initialInterval":     "1000ms", // 1s
				"randomizationFactor": 1.0,
				"multiplier":          2.0,
				"maxInterval":         "120s", // 2m
				"maxElapsedTime":      "30m",  // 30m
			},
			overrides: func(config *retry.Config) {
				config.Policy = retry.PolicyExponential
				config.InitialInterval = 1 * time.Second
				config.RandomizationFactor = 1.0
				config.Multiplier = 2.0
				config.MaxInterval = 2 * time.Minute
				config.MaxElapsedTime = 30 * time.Minute
			},
			err: "",
		},
	}

	for name, tc := range tests {
		t.Run(name, func(t *testing.T) {
			actual, err := DecodeConfig(tc.config)
			if tc.err != "" {
				if assert.Error(t, err) {
					assert.Equal(t, tc.err, err.Error())
				}
			} else {
				config := retry.DefaultConfig
				if tc.overrides != nil {
					tc.overrides(&config)
				}
				assert.Equal(t, config, actual, "unexpected decoded configuration")
			}
		})
	}
}
