/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package runtime_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/resiliency/retry"
	"github.com/nanobus/nanobus/pkg/runtime"
)

func TestRetryDecode(t *testing.T) {
	tests := map[string]struct {
		config    runtime.Backoff
		overrides func(config *retry.Config)
		err       string
	}{
		"invalid policy type": {
			config:    runtime.Backoff{},
			overrides: nil,
			err:       "1 error(s) decoding:\n\n* error decoding 'policy': invalid PolicyType \"invalid\": unexpected back off policy type: invalid",
		},
		"constant default": {
			config: runtime.Backoff{
				Constant: &runtime.ConstantBackoff{},
			},
			overrides: nil,
			err:       "",
		},
		"constant with duration": {
			config: runtime.Backoff{
				Constant: &runtime.ConstantBackoff{
					Duration: runtime.Duration(time.Second * 10),
				},
			},
			overrides: func(config *retry.Config) {
				config.Duration = 10 * time.Second
			},
			err: "",
		},
		"exponential default": {
			config: runtime.Backoff{
				Exponential: &runtime.ExponentialBackoff{},
			},
			overrides: func(config *retry.Config) {
				config.Policy = retry.PolicyExponential
			},
			err: "",
		},
		"exponential": {
			config: runtime.Backoff{
				Exponential: &runtime.ExponentialBackoff{
					InitialInterval:     runtime.Duration(time.Second * 1), // 1s
					RandomizationFactor: 1.0,
					Multiplier:          2.0,
					MaxInterval:         runtime.Duration(time.Minute * 2),  // 2m
					MaxElapsedTime:      runtime.Duration(time.Minute * 30), // 30m,
				},
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
			actual, err := runtime.ConvertBackoffConfig(tc.config)
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
