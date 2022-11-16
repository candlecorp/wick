/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core

import (
	"context"
	"errors"
	"io"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/stream"
)

type ChannelTestConfig struct {
}

// ChannelTest is the NamedLoader for the channel test action.
func ChannelTest() (string, actions.Loader) {
	return "channel_test", ChannelTestLoader
}

func ChannelTestLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := ChannelTestConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return ChannelTestAction(&c), nil
}

func ChannelTestAction(
	config *ChannelTestConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		source, ok := stream.SourceFromContext(ctx)
		if !ok {
			return nil, errors.New("stream not in context")
		}
		sink, ok := stream.SinkFromContext(ctx)
		if !ok {
			return nil, errors.New("stream not in context")
		}

		input, _ := data["input"].(map[string]interface{})
		prefix, _ := input["prefix"].(string)

		var in map[string]interface{}
		for {
			if err := source.Next(&in, nil); err != nil {
				if err == io.EOF {
					break
				}
				return nil, err
			}

			out := make(map[string]interface{}, len(in))
			for k, v := range in {
				switch v := v.(type) {
				case string:
					out[k] = prefix + v
				default:
					out[k] = v
				}
			}

			if err := sink.Next(out, nil); err != nil {
				return nil, err
			}
		}

		return nil, nil
	}
}
