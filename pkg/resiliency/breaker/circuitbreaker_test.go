/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package breaker_test

import (
	"errors"
	"testing"
	"time"

	"github.com/go-logr/logr"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resiliency/breaker"
)

func TestCircuitBreaker(t *testing.T) {
	var trip expr.ValueExpr
	err := trip.DecodeString("consecutiveFailures > 2")
	require.NoError(t, err)
	cb := breaker.CircuitBreaker{
		Name:    "test",
		Trip:    &trip,
		Timeout: 10 * time.Millisecond,
	}
	log := logr.Discard()
	cb.Initialize(log)
	for i := 0; i < 3; i++ {
		cb.Execute(func() error {
			return errors.New("test")
		})
	}
	err = cb.Execute(func() error {
		return nil
	})
	assert.EqualError(t, err, "circuit breaker is open")
	time.Sleep(100 * time.Millisecond)
	err = cb.Execute(func() error {
		return nil
	})
	assert.NoError(t, err)
}
