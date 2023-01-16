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

	"gocloud.dev/blob/s3blob"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type S3BlobConfig struct {
	Region     string `mapstructure:"region" validate:"required"`
	BucketName string `mapstructure:"bucketName" validate:"required"`
}

// S3Blob is the NamedLoader for a filesystem blob.
func S3Blob() (string, resource.Loader) {
	return "nanobus.resource.s3blob/v1", S3BlobLoader
}

func S3BlobLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c S3BlobConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	// Establish an AWS session.
	// See https://docs.aws.amazon.com/sdk-for-go/api/aws/session/ for more info.
	// The region must match the region for "my-bucket".
	sess, err := session.NewSession(&aws.Config{
		Region: aws.String(c.Region),
	})
	if err != nil {
		return nil, err
	}

	// Create a *blob.Bucket.
	return s3blob.OpenBucket(ctx, sess, c.BucketName, &s3blob.Options{})
}
