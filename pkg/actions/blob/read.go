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
	"fmt"

	"github.com/cenkalti/backoff/v4"
	"gocloud.dev/blob"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func ReadLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := ReadConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	var codecs codec.Codecs
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources,
		"codec:lookup", &codecs); err != nil {
		return nil, err
	}

	codec, ok := codecs[string(c.Codec)]
	if !ok {
		return nil, fmt.Errorf("unknown codec %q", c.Codec)
	}

	bucket, err := resource.Get[*blob.Bucket](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return ReadAction(bucket, codec, &c), nil
}

func ReadAction(
	bucket *blob.Bucket,
	codec codec.Codec,
	config *ReadConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		keyInt, err := config.Key.Eval(data)
		if err != nil {
			return nil, backoff.Permanent(fmt.Errorf("could not evaluate key: %w", err))
		}
		key := fmt.Sprintf("%v", keyInt)

		dataBytes, err := bucket.ReadAll(ctx, key)
		if err != nil {
			return nil, fmt.Errorf("could not read key: %w", err)
		}

		result, _, err := codec.Decode(dataBytes, config.CodecArgs...)

		return result, err
	}
}
