/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package errorz

import (
	"encoding/json"
	"fmt"
)

// The code below is based on gRPC.

// ErrCode is an RPC error code.
type ErrCode int

const (
	// OK indicates the operation was successful.
	OK ErrCode = 0

	// Canceled indicates the operation was canceled (typically by the caller).
	Canceled ErrCode = 1

	// Unknown error. An example of where this error may be returned is
	// if a Status value received from another address space belongs to
	// an error-space that is not known in this address space. Also
	// errors raised by APIs that do not return enough error information
	// may be converted to this error.
	Unknown ErrCode = 2

	// InvalidArgument indicates client specified an invalid argument.
	// Note that this differs from FailedPrecondition. It indicates arguments
	// that are problematic regardless of the state of the system
	// (e.g., a malformed file name).
	InvalidArgument ErrCode = 3

	// DeadlineExceeded means operation expired before completion.
	// For operations that change the state of the system, this error may be
	// returned even if the operation has completed successfully. For
	// example, a successful response from a server could have been delayed
	// long enough for the deadline to expire.
	DeadlineExceeded ErrCode = 4

	// NotFound means some requested entity (e.g., file or directory) was
	// not found.
	NotFound ErrCode = 5

	// AlreadyExists means an attempt to create an entity failed because one
	// already exists.
	AlreadyExists ErrCode = 6

	// PermissionDenied indicates the caller does not have permission to
	// execute the specified operation. It must not be used for rejections
	// caused by exhausting some resource (use ResourceExhausted
	// instead for those errors). It must not be
	// used if the caller cannot be identified (use Unauthenticated
	// instead for those errors).
	PermissionDenied ErrCode = 7

	// ResourceExhausted indicates some resource has been exhausted, perhaps
	// a per-user quota, or perhaps the entire file system is out of space.
	ResourceExhausted ErrCode = 8

	// FailedPrecondition indicates operation was rejected because the
	// system is not in a state required for the operation's execution.
	// For example, directory to be deleted may be non-empty, an rmdir
	// operation is applied to a non-directory, etc.
	//
	// A litmus test that may help a service implementor in deciding
	// between FailedPrecondition, Aborted, and Unavailable:
	//  (a) Use Unavailable if the client can retry just the failing call.
	//  (b) Use Aborted if the client should retry at a higher-level
	//      (e.g., restarting a read-modify-write sequence).
	//  (c) Use FailedPrecondition if the client should not retry until
	//      the system state has been explicitly fixed. E.g., if an "rmdir"
	//      fails because the directory is non-empty, FailedPrecondition
	//      should be returned since the client should not retry unless
	//      they have first fixed up the directory by deleting files from it.
	//  (d) Use FailedPrecondition if the client performs conditional
	//      Get/Update/Delete on a resource and the resource on the
	//      server does not match the condition. E.g., conflicting
	//      read-modify-write on the same resource.
	FailedPrecondition ErrCode = 9

	// Aborted indicates the operation was aborted, typically due to a
	// concurrency issue like sequencer check failures, transaction aborts,
	// etc.
	//
	// See litmus test above for deciding between FailedPrecondition,
	// Aborted, and Unavailable.
	Aborted ErrCode = 10

	// OutOfRange means operation was attempted past the valid range.
	// E.g., seeking or reading past end of file.
	//
	// Unlike InvalidArgument, this error indicates a problem that may
	// be fixed if the system state changes. For example, a 32-bit file
	// system will generate InvalidArgument if asked to read at an
	// offset that is not in the range [0,2^32-1], but it will generate
	// OutOfRange if asked to read from an offset past the current
	// file size.
	//
	// There is a fair bit of overlap between FailedPrecondition and
	// OutOfRange. We recommend using OutOfRange (the more specific
	// error) when it applies so that callers who are iterating through
	// a space can easily look for an OutOfRange error to detect when
	// they are done.
	OutOfRange ErrCode = 11

	// Unimplemented indicates operation is not implemented or not
	// supported/enabled in this service.
	Unimplemented ErrCode = 12

	// Internal errors. Means some invariants expected by underlying
	// system has been broken. If you see one of these errors,
	// something is very broken.
	Internal ErrCode = 13

	// Unavailable indicates the service is currently unavailable.
	// This is a most likely a transient condition and may be corrected
	// by retrying with a backoff. Note that it is not always safe to retry
	// non-idempotent operations.
	//
	// See litmus test above for deciding between FailedPrecondition,
	// Aborted, and Unavailable.
	Unavailable ErrCode = 14

	// DataLoss indicates unrecoverable data loss or corruption.
	DataLoss ErrCode = 15

	// Unauthenticated indicates the request does not have valid
	// authentication credentials for the operation.
	Unauthenticated ErrCode = 16
)

// String returns the string representation of c.
func (c ErrCode) String() string {
	if int(c) >= numCodeNames {
		return "unknown"
	}
	return codeNames[c]
}

// HTTPStatus reports a suitable HTTP status code for an error, based on its code.
// If err is nil it reports 200. If it's not an *Error it reports 500.
func (c ErrCode) HTTPStatus() int {
	if int(c) >= numCodeStatus {
		return 500
	}
	return codeStatus[c]
}

func (c ErrCode) MarshalJSON() ([]byte, error) {
	s := c.String()
	return []byte("\"" + s + "\""), nil
}

func (t *ErrCode) UnmarshalJSON(data []byte) error {
	var str string
	if err := json.Unmarshal(data, &str); err != nil {
		return err
	}
	return t.Parse(str)
}

func (t *ErrCode) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}
	return t.Parse(str)
}

func (t *ErrCode) Parse(str string) error {
	code, ok := CodeLookup[str]
	if !ok {
		return fmt.Errorf("unknown error code %q", str)
	}
	*t = code

	return nil
}

var numCodeNames = len(codeNames)

var CodeLookup = map[string]ErrCode{
	"ok":                  OK,
	"canceled":            Canceled,
	"unknown":             Unknown,
	"invalid_argument":    InvalidArgument,
	"deadline_exceeded":   DeadlineExceeded,
	"not_found":           NotFound,
	"already_exists":      AlreadyExists,
	"permission_denied":   PermissionDenied,
	"resource_exhausted":  ResourceExhausted,
	"failed_precondition": FailedPrecondition,
	"aborted":             Aborted,
	"out_of_range":        OutOfRange,
	"unimplemented":       Unimplemented,
	"internal":            Internal,
	"unavailable":         Unavailable,
	"data_loss":           DataLoss,
	"unauthenticated":     Unauthenticated,
}

var codeNames = [...]string{
	OK:                 "ok",
	Canceled:           "canceled",
	Unknown:            "unknown",
	InvalidArgument:    "invalid_argument",
	DeadlineExceeded:   "deadline_exceeded",
	NotFound:           "not_found",
	AlreadyExists:      "already_exists",
	PermissionDenied:   "permission_denied",
	ResourceExhausted:  "resource_exhausted",
	FailedPrecondition: "failed_precondition",
	Aborted:            "aborted",
	OutOfRange:         "out_of_range",
	Unimplemented:      "unimplemented",
	Internal:           "internal",
	Unavailable:        "unavailable",
	DataLoss:           "data_loss",
	Unauthenticated:    "unauthenticated",
}

var numCodeStatus = len(codeStatus)

var codeStatus = [...]int{
	OK:                 200,
	Canceled:           499,
	Unknown:            500,
	InvalidArgument:    400,
	DeadlineExceeded:   504,
	NotFound:           404,
	AlreadyExists:      409,
	PermissionDenied:   403,
	ResourceExhausted:  429,
	FailedPrecondition: 400,
	Aborted:            409,
	OutOfRange:         400,
	Unimplemented:      501,
	Internal:           500,
	Unavailable:        503,
	DataLoss:           500,
	Unauthenticated:    401,
}
