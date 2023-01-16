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

func WriteLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := WriteConfig{}
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

	return WriteAction(bucket, codec, &c), nil
}

func WriteAction(
	bucket *blob.Bucket,
	codec codec.Codec,
	config *WriteConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		keyInt, err := config.Key.Eval(data)
		if err != nil {
			return nil, backoff.Permanent(fmt.Errorf("could not evaluate key: %w", err))
		}
		key := fmt.Sprintf("%v", keyInt)

		writer, err := bucket.NewWriter(ctx, key, nil)
		if err != nil {
			return nil, fmt.Errorf("could not evaluate key: %w", err)
		}
		defer writer.Close()

		if s, ok := stream.SourceFromContext(ctx); ok {
			for {
				var record any
				if err := s.Next(&record, nil); err != nil {
					if err == io.EOF {
						return nil, nil
					}

					return nil, err
				}

				dataBytes, err := codec.Encode(record, config.CodecArgs...)
				if err != nil {
					return nil, err
				}

				if _, err = writer.Write(dataBytes); err != nil {
					return nil, err
				}
			}
		} else {
			var d any = data["input"]
			if config.Data != nil {
				if d, err = config.Data.Eval(data); err != nil {
					return nil, backoff.Permanent(fmt.Errorf("could not evaluate data: %w", err))
				}
			}

			dataBytes, err := codec.Encode(d, config.CodecArgs...)
			if err != nil {
				return nil, backoff.Permanent(fmt.Errorf("could not encode data: %w", err))
			}

			if _, err = writer.Write(dataBytes); err != nil {
				return nil, err
			}
		}

		return nil, nil
	}
}
