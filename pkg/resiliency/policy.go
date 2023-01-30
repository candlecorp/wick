/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package resiliency

import (
	"context"
	"errors"
	"time"

	"github.com/cenkalti/backoff/v4"
	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/pkg/resiliency/breaker"
	"github.com/nanobus/nanobus/pkg/resiliency/retry"
)

type (
	// Operation represents a function to invoke with resiliency policies applied.
	Operation func(ctx context.Context) error

	// Runner represents a function to invoke `oper` with resiliency policies applied.
	Runner func(ctx context.Context, oper Operation) error
)

// Policy returns a policy runner that encapsulates the configured
// resiliency policies in a simple execution wrapper.
func Policy(log logr.Logger, operationName string, t time.Duration, r *retry.Config, cb *breaker.CircuitBreaker) Runner {
	return func(ctx context.Context, oper Operation) error {
		operation := oper
		if t > 0 {
			// Handle timeout
			operCopy := operation
			operation = func(ctx context.Context) error {
				ctx, cancel := context.WithTimeout(ctx, t)
				defer cancel()

				done := make(chan error, 1)
				go func() {
					done <- operCopy(ctx)
				}()

				select {
				case err := <-done:
					return err
				case <-ctx.Done():
					return ctx.Err()
				}
			}
		}

		if cb != nil {
			operCopy := operation
			operation = func(ctx context.Context) error {
				err := cb.Execute(func() error {
					return operCopy(ctx)
				})
				if r != nil && breaker.IsErrorPermanent(err) {
					// Break out of retry.
					err = backoff.Permanent(err)
				}
				return err
			}
		}

		if r == nil {
			return operation(ctx)
		}

		// Use retry/back off
		b := r.NewBackOffWithContext(ctx)
		err := retry.NotifyRecover(func() error {
			err := operation(ctx)
			if err != nil {
				var perm *backoff.PermanentError
				if errors.As(err, &perm) {
					return err
				}
				var retriable *RetriableError
				if errors.As(err, &retriable) {
					return retriable.Err
				}

				// By default, errors are permanent errors
				// unless wrapped by RetriableError.
				err = backoff.Permanent(err)
			}
			return err
		}, b, func(err error, _ time.Duration) {
			log.Error(err, "Error processing operation. Retrying...", "operation", operationName)
		}, func() {
			log.Info("Recovered processing operation.", "operation", operationName)
		})
		if err != nil {
			var perr *backoff.PermanentError
			if errors.As(err, &perr) {
				err = perr.Err
			}
		}
		return err
	}
}
