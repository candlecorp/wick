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

	"gocloud.dev/blob"
	_ "gocloud.dev/blob/azureblob"
	_ "gocloud.dev/blob/fileblob"
	_ "gocloud.dev/blob/gcsblob"
	_ "gocloud.dev/blob/memblob"
	_ "gocloud.dev/blob/s3blob"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type URLBlobConfig struct {
	URL string `mapstructure:"url"`
}

// Connection is the NamedLoader for a postgres connection.
func URLBlob() (string, resource.Loader) {
	return "nanobus.resource.urlblob/v1", URLBlobLoader
}

func URLBlobLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c URLBlobConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	bucket, err := blob.OpenBucket(ctx, c.URL)

	return bucket, err
}
