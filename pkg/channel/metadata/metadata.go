/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package metadata

import (
	"context"
	"fmt"
	"strings"
)

// MD is a mapping from metadata keys to values. Users should use the following
// two convenience functions New and Pairs to generate MD.
type MD map[string][]string

// New creates an MD from a given key-value map.
//
// Only the following ASCII characters are allowed in keys:
//   - digits: 0-9
//   - uppercase letters: A-Z (normalized to lower)
//   - lowercase letters: a-z
//   - special characters: -_.
//
// Uppercase letters are automatically converted to lowercase.
func New(m map[string]string) MD {
	md := MD{}
	for k, val := range m {
		key := strings.ToLower(k)
		md[key] = append(md[key], val)
	}
	return md
}

// Pairs returns an MD formed by the mapping of key, value ...
// Pairs panics if len(kv) is odd.
//
// Only the following ASCII characters are allowed in keys:
//   - digits: 0-9
//   - uppercase letters: A-Z (normalized to lower)
//   - lowercase letters: a-z
//   - special characters: -_.
//
// Uppercase letters are automatically converted to lowercase.
func Pairs(kv ...string) MD {
	if len(kv)%2 == 1 {
		panic(fmt.Sprintf("metadata: Pairs got the odd number of input pairs for metadata: %d", len(kv)))
	}
	md := MD{}
	for i := 0; i < len(kv); i += 2 {
		key := strings.ToLower(kv[i])
		md[key] = append(md[key], kv[i+1])
	}
	return md
}

// Len returns the number of items in md.
func (md MD) Len() int {
	return len(md)
}

// Copy returns a copy of md.
func (md MD) Copy() MD {
	return Join(md)
}

// Get obtains the values for a given key.
func (md MD) Get(k string) []string {
	k = strings.ToLower(k)
	return md[k]
}

// Get obtains the values for a given key.
func (md MD) Scalar(k string) (string, bool) {
	k = strings.ToLower(k)
	values := md[k]
	if len(values) > 0 {
		return values[0], true
	}
	return "", false
}

// Set sets the value of a given key with a slice of values.
func (md MD) Set(k string, vals ...string) {
	if len(vals) == 0 {
		return
	}
	k = strings.ToLower(k)
	md[k] = vals
}

// Append adds the values to key k, not overwriting what was already stored at that key.
func (md MD) Append(k string, vals ...string) {
	if len(vals) == 0 {
		return
	}
	k = strings.ToLower(k)
	md[k] = append(md[k], vals...)
}

// Join joins any number of mds into a single MD.
// The order of values for each key is determined by the order in which
// the mds containing those values are presented to Join.
func Join(mds ...MD) MD {
	out := MD{}
	for _, md := range mds {
		for k, v := range md {
			out[k] = append(out[k], v...)
		}
	}
	return out
}

type mdIncomingKey struct{}
type mdOutgoingKey struct{}

// NewIncomingContext creates a new context with incoming md attached.
func NewIncomingContext(ctx context.Context, md MD) context.Context {
	return context.WithValue(ctx, mdIncomingKey{}, md)
}

// NewOutgoingContext creates a new context with outgoing md attached. If used
// in conjunction with AppendToOutgoingContext, NewOutgoingContext will
// overwrite any previously-appended metadata.
func NewOutgoingContext(ctx context.Context, md MD) context.Context {
	return context.WithValue(ctx, mdOutgoingKey{}, rawMD{md: md})
}

// AppendToOutgoingContext returns a new context with the provided kv merged
// with any existing metadata in the context. Please refer to the
// documentation of Pairs for a description of kv.
func AppendToOutgoingContext(ctx context.Context, kv ...string) context.Context {
	if len(kv)%2 == 1 {
		panic(fmt.Sprintf("metadata: AppendToOutgoingContext got an odd number of input pairs for metadata: %d", len(kv)))
	}
	md, _ := ctx.Value(mdOutgoingKey{}).(rawMD)
	added := make([][]string, len(md.added)+1)
	copy(added, md.added)
	added[len(added)-1] = make([]string, len(kv))
	copy(added[len(added)-1], kv)
	return context.WithValue(ctx, mdOutgoingKey{}, rawMD{md: md.md, added: added})
}

// FromIncomingContext returns the incoming metadata in ctx if it exists.  The
// returned MD should not be modified. Writing to it may cause races.
// Modification should be made to copies of the returned MD.
func FromIncomingContext(ctx context.Context) (md MD, ok bool) {
	md, ok = ctx.Value(mdIncomingKey{}).(MD)
	return
}

// FromOutgoingContextRaw returns the un-merged, intermediary contents
// of rawMD. Remember to perform strings.ToLower on the keys. The returned
// MD should not be modified. Writing to it may cause races. Modification
// should be made to copies of the returned MD.
//
// This is intended for gRPC-internal use ONLY.
func FromOutgoingContextRaw(ctx context.Context) (MD, [][]string, bool) {
	raw, ok := ctx.Value(mdOutgoingKey{}).(rawMD)
	if !ok {
		return nil, nil, false
	}

	return raw.md, raw.added, true
}

// FromOutgoingContext returns the outgoing metadata in ctx if it exists.  The
// returned MD should not be modified. Writing to it may cause races.
// Modification should be made to copies of the returned MD.
func FromOutgoingContext(ctx context.Context) (MD, bool) {
	raw, ok := ctx.Value(mdOutgoingKey{}).(rawMD)
	if !ok {
		return nil, false
	}

	out := raw.md.Copy()
	for _, added := range raw.added {
		if len(added)%2 == 1 {
			panic(fmt.Sprintf("metadata: FromOutgoingContext got an odd number of input pairs for metadata: %d", len(added)))
		}

		for i := 0; i < len(added); i += 2 {
			key := strings.ToLower(added[i])
			out[key] = append(out[key], added[i+1])
		}
	}
	return out, ok
}

type rawMD struct {
	md    MD
	added [][]string
}
