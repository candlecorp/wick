/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package resiliency_test

import (
	"context"
	"testing"
	"time"

	"github.com/go-logr/logr"
	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resiliency/breaker"
	"github.com/nanobus/nanobus/pkg/resiliency/retry"
)

func TestPolicy(t *testing.T) {
	retryValue := retry.DefaultConfig
	cbValue := breaker.CircuitBreaker{
		Name:     "test",
		Interval: 10 * time.Millisecond,
		Timeout:  10 * time.Millisecond,
	}
	log := logr.Discard()
	cbValue.Initialize(log)
	tests := map[string]struct {
		t  time.Duration
		r  *retry.Config
		cb *breaker.CircuitBreaker
	}{
		"empty": {},
		"all": {
			t:  10 * time.Millisecond,
			r:  &retryValue,
			cb: &cbValue,
		},
	}

	ctx := context.Background()
	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			called := false
			fn := func(ctx context.Context) error {
				called = true

				return nil
			}
			policy := resiliency.Policy(logr.Discard(), name, tt.t, tt.r, tt.cb)
			policy(ctx, fn)
			assert.True(t, called)
		})
	}
}
