/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package blob

import (
	"context"

	"gocloud.dev/blob/memblob"

	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type MemBlobConfig struct {
}

// MemBlob is the NamedLoader for an in-memory blob.
func MemBlob() (string, resource.Loader) {
	return "nanobus.resource.memblob/v1", MemBlobLoader
}

func MemBlobLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	// Create a *blob.Bucket.
	return memblob.OpenBucket(&memblob.Options{}), nil
}
