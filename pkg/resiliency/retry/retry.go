/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package retry

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/cenkalti/backoff/v4"

	"github.com/nanobus/nanobus/pkg/config"
)

// PolicyType denotes if the back off delay should be constant or exponential.
type PolicyType int

const (
	// PolicyConstant is a backoff policy that always returns the same backoff delay.
	PolicyConstant PolicyType = iota
	// PolicyExponential is a backoff implementation that increases the backoff period
	// for each retry attempt using a randomization function that grows exponentially.
	PolicyExponential
)

// Config encapsulates the back off policy configuration.
type Config struct {
	Policy PolicyType `mapstructure:"policy"`

	// Constant back off
	Duration time.Duration `mapstructure:"duration"`

	// Exponential back off
	InitialInterval     time.Duration `mapstructure:"initialInterval"`
	RandomizationFactor float32       `mapstructure:"randomizationFactor"`
	Multiplier          float32       `mapstructure:"multiplier"`
	MaxInterval         time.Duration `mapstructure:"maxInterval"`
	MaxElapsedTime      time.Duration `mapstructure:"maxElapsedTime"`

	// Additional options
	MaxRetries int64 `mapstructure:"maxRetries"`
}

// DefaultConfig represents the default configuration for a
// `RetryConfig`.
var DefaultConfig = Config{
	Policy:              PolicyConstant,
	Duration:            5 * time.Second,
	InitialInterval:     backoff.DefaultInitialInterval,
	RandomizationFactor: backoff.DefaultRandomizationFactor,
	Multiplier:          backoff.DefaultMultiplier,
	MaxInterval:         backoff.DefaultMaxInterval,
	MaxElapsedTime:      backoff.DefaultMaxElapsedTime,
	MaxRetries:          -1,
}

// DecodeConfig decodes a Go struct into a `BackOffConfig`.
func DecodeConfig(input interface{}) (Config, error) {
	c := DefaultConfig
	err := config.Decode(input, &c)

	return c, err
}

// NewBackOff returns a BackOff instance for use with `RetryNotifyRecover`
// or `backoff.RetryNotify` directly. The instance will not stop due to
// context cancellation. To support cancellation (recommended), use
// `NewBackOffWithContext`.
//
// Since the underlying backoff implementations are not always thread safe,
// `NewBackOff` or `NewBackOffWithContext` should be called each time
// `RetryNotifyRecover` or `backoff.RetryNotify` is used.
func (c *Config) NewBackOff() backoff.BackOff {
	var b backoff.BackOff
	switch c.Policy {
	case PolicyConstant:
		b = backoff.NewConstantBackOff(c.Duration)
	case PolicyExponential:
		eb := backoff.NewExponentialBackOff()
		eb.InitialInterval = c.InitialInterval
		eb.RandomizationFactor = float64(c.RandomizationFactor)
		eb.Multiplier = float64(c.Multiplier)
		eb.MaxInterval = c.MaxInterval
		eb.MaxElapsedTime = c.MaxElapsedTime
		b = eb
	}

	if c.MaxRetries >= 0 {
		b = backoff.WithMaxRetries(b, uint64(c.MaxRetries))
	}

	return b
}

// NewBackOffWithContext returns a BackOff instance for use with `RetryNotifyRecover`
// or `backoff.RetryNotify` directly. The provided context is used to cancel retries
// if it is canceled.
//
// Since the underlying backoff implementations are not always thread safe,
// `NewBackOff` or `NewBackOffWithContext` should be called each time
// `RetryNotifyRecover` or `backoff.RetryNotify` is used.
func (c *Config) NewBackOffWithContext(ctx context.Context) backoff.BackOff {
	b := c.NewBackOff()

	return backoff.WithContext(b, ctx)
}

// NotifyRecover is a wrapper around backoff.RetryNotify that adds another callback for when an operation
// previously failed but has since recovered. The main purpose of this wrapper is to call `notify` only when
// the operations fails the first time and `recovered` when it finally succeeds. This can be helpful in limiting
// log messages to only the events that operators need to be alerted on.
func NotifyRecover(operation backoff.Operation, b backoff.BackOff, notify backoff.Notify, recovered func()) error {
	var notified bool

	return backoff.RetryNotify(func() error {
		err := operation()
		if err != nil && strings.HasPrefix(err.Error(), "not_found") {
			err = backoff.Permanent(err)
		}

		if err == nil && notified {
			notified = false
			recovered()
		}

		return err
	}, b, func(err error, d time.Duration) {
		if !notified {
			notify(err, d)
			notified = true
		}
	})
}

// FromString handles converting a string value to `p`.
func (p *PolicyType) FromString(value string) error {
	switch strings.ToLower(value) {
	case "constant":
		*p = PolicyConstant
		return nil
	case "exponential":
		*p = PolicyExponential
		return nil
	}

	return fmt.Errorf("unexpected back off policy type: %s", value)
}
