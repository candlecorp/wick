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

	"gocloud.dev/blob/fileblob"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type FSBlobConfig struct {
	Dir    string `mapstructure:"dir" validate:"required"`
	Create bool   `mapstructure:"create"`
}

// FSBlob is the NamedLoader for a filesystem blob.
func FSBlob() (string, resource.Loader) {
	return "nanobus.resource.fsblob/v1", FSBlobLoader
}

func FSBlobLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c FSBlobConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	// Create a *blob.Bucket.
	return fileblob.OpenBucket(c.Dir, &fileblob.Options{
		CreateDir: c.Create,
	})
}
