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

	"gocloud.dev/blob/gcsblob"
	"gocloud.dev/gcp"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type GCSBlobConfig struct {
	BucketName     string `mapstructure:"bucketName" validate:"required"`
	GoogleAccessID string `mapstructure:"googleAccessID" validate:"required"`
}

// GCSBlob is the NamedLoader for a GCP blob.
func GCSBlob() (string, resource.Loader) {
	return "nanobus.resource.gcsblob/v1", GCSBlobLoader
}

func GCSBlobLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c GCSBlobConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	// Your GCP credentials.
	// See https://cloud.google.com/docs/authentication/production
	// for more info on alternatives.
	creds, err := gcp.DefaultCredentials(ctx)
	if err != nil {
		return nil, err
	}

	// Create an HTTP client.
	// This example uses the default HTTP transport and the credentials
	// created above.
	client, err := gcp.NewHTTPClient(
		gcp.DefaultTransport(),
		gcp.CredentialsTokenSource(creds))
	if err != nil {
		return nil, err
	}

	// Create a *blob.Bucket.
	return gcsblob.OpenBucket(ctx, client, c.BucketName, &gcsblob.Options{
		GoogleAccessID: c.GoogleAccessID,
	})
}
