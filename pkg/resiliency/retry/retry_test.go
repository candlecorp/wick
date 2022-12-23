/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package retry_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/resiliency/retry"
)

var errRetry = errors.New("Testing")

func TestRetryNotifyRecoverMaxRetries(t *testing.T) {
	config := retry.DefaultConfig
	config.MaxRetries = 3
	config.Duration = 1

	var operationCalls, notifyCalls, recoveryCalls int

	b := config.NewBackOff()
	err := retry.NotifyRecover(func() error {
		operationCalls++

		return errRetry
	}, b, func(err error, d time.Duration) {
		notifyCalls++
	}, func() {
		recoveryCalls++
	})

	assert.Error(t, err)
	assert.Equal(t, errRetry, err)
	assert.Equal(t, 4, operationCalls)
	assert.Equal(t, 1, notifyCalls)
	assert.Equal(t, 0, recoveryCalls)
}

func TestRetryNotifyRecoverRecovery(t *testing.T) {
	config := retry.DefaultConfig
	config.MaxRetries = 3
	config.Duration = 1

	var operationCalls, notifyCalls, recoveryCalls int

	b := config.NewBackOff()
	err := retry.NotifyRecover(func() error {
		operationCalls++

		if operationCalls >= 2 {
			return nil
		}

		return errRetry
	}, b, func(err error, d time.Duration) {
		notifyCalls++
	}, func() {
		recoveryCalls++
	})

	assert.NoError(t, err)
	assert.Equal(t, 2, operationCalls)
	assert.Equal(t, 1, notifyCalls)
	assert.Equal(t, 1, recoveryCalls)
}

func TestRetryNotifyRecoverCancel(t *testing.T) {
	config := retry.DefaultConfig
	config.Policy = retry.PolicyConstant
	config.Duration = 10 * time.Second

	var notifyCalls, recoveryCalls int

	ctx, cancel := context.WithCancel(context.Background())
	b := config.NewBackOffWithContext(ctx)
	errC := make(chan error, 1)
	startedC := make(chan struct{}, 1)

	go func() {
		errC <- retry.NotifyRecover(func() error {
			return errRetry
		}, b, func(err error, d time.Duration) {
			notifyCalls++
			startedC <- struct{}{}
		}, func() {
			recoveryCalls++
		})
	}()

	<-startedC
	cancel()

	err := <-errC
	assert.Error(t, err)
	assert.True(t, errors.Is(err, context.Canceled))
	assert.Equal(t, 1, notifyCalls)
	assert.Equal(t, 0, recoveryCalls)
}
