/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package resiliency

// RetriableError signals that the operation can be retried.
type RetriableError struct {
	Err error
}

func (e *RetriableError) Error() string {
	return e.Err.Error()
}

func (e *RetriableError) Unwrap() error {
	return e.Err
}

func (e *RetriableError) Is(target error) bool {
	_, ok := target.(*RetriableError)
	return ok
}

// Retriable wraps the given err in a *RetriableError.
func Retriable(err error) error {
	if err == nil {
		return nil
	}
	return &RetriableError{
		Err: err,
	}
}
