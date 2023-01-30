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

	"github.com/go-logr/logr"
	"gocloud.dev/blob"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/stream"
)

func WriteLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := WriteConfig{
		Codec: "bytes",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var log logr.Logger
	var resources resource.Resources
	var codecs codec.Codecs
	if err := resolve.Resolve(resolver,
		"system:logger", &log,
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

	return WriteAction(log, bucket, codec, &c), nil
}

func WriteAction(
	log logr.Logger,
	bucket *blob.Bucket,
	codec codec.Codec,
	config *WriteConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		key, err := expr.EvalAsStringE(config.Key, data)
		if err != nil {
			return nil, fmt.Errorf("could not evaluate key: %w", err)
		}

		// Note: writer.Close seems to fail due to "context closed"
		// if `ctx` is used. Even if the context is not done proir.
		writeCtx, cancelWrite := context.WithCancel(context.Background())
		defer cancelWrite()

		writer, err := bucket.NewWriter(writeCtx, key, nil)
		if err != nil {
			return nil, resiliency.Retriable(fmt.Errorf("could create writer: %w", err))
		}
		defer writer.Close()

		if s, ok := stream.SourceFromContext(ctx); ok {
			first := true
			for {
				if !first {
					if config.DelimiterString != nil {
						if _, err := writer.Write([]byte(*config.DelimiterString)); err != nil {
							return nil, err
						}
					} else if config.DelimiterBytes != nil {
						if _, err := writer.Write(config.DelimiterBytes); err != nil {
							return nil, err
						}
					}
				}

				first = false
				var record any
				if err := s.Next(&record, nil); err != nil {
					if err == io.EOF {
						if err = writer.Close(); err != nil {
							return nil, err
						}

						return nil, nil
					}

					cancelWrite()
					writer.Close()
					return nil, err
				}

				dataBytes, err := codec.Encode(record, config.CodecArgs...)
				if err != nil {
					return nil, err
				}

				if _, err := writer.Write(dataBytes); err != nil {
					return nil, err
				}
			}
		} else {
			var d any = data["input"]
			if config.Data != nil {
				if d, err = config.Data.Eval(data); err != nil {
					return nil, fmt.Errorf("could not evaluate data: %w", err)
				}
			}

			dataBytes, err := codec.Encode(d, config.CodecArgs...)
			if err != nil {
				return nil, fmt.Errorf("could not encode data: %w", err)
			}

			if _, err = writer.Write(dataBytes); err != nil {
				return nil, resiliency.Retriable(err)
			}
		}

		return nil, nil
	}
}
