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
	"errors"
	"fmt"
	"io"

	"github.com/cenkalti/backoff/v4"
	"gocloud.dev/blob"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/stream"
)

func ReadLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := ReadConfig{
		Codec:      "bytes",
		BufferSize: 1024,
	}
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
		if isNil(keyInt) {
			return nil, backoff.Permanent(errors.New("key is nil"))
		}
		key := fmt.Sprintf("%v", keyInt)

		s, _ := stream.SinkFromContext(ctx)
		if s != nil {
			reader, err := bucket.NewReader(ctx, key, nil)
			if err != nil {
				return nil, fmt.Errorf("could not open reader for key %s: %w", key, err)
			}
			defer reader.Close()

			for {
				buf := make([]byte, config.BufferSize)
				n, err := reader.Read(buf)
				if err != nil {
					if err == io.EOF {
						s.Complete()
						return nil, nil
					}
					return nil, err
				}

				if n == 0 {
					return nil, nil
				}

				if err := s.Next(buf[0:n], nil); err != nil {
					s.Error(err)
					return nil, err
				}

				if uint32(n) < config.BufferSize {
					return nil, nil
				}
			}
		} else {
			dataBytes, err := bucket.ReadAll(ctx, key)
			if err != nil {
				return nil, fmt.Errorf("could not read key: %w", err)
			}

			result, _, err := codec.Decode(dataBytes, config.CodecArgs...)

			return result, err
		}
	}
}
