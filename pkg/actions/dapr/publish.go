/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package dapr

import (
	"context"
	"fmt"

	dapr "github.com/dapr/go-sdk/client"
	"go.opentelemetry.io/otel"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/telemetry/tracing"
)

func PublishLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := PublishConfig{
		Resource:         "dapr",
		Codec:            "json",
		PropogateTracing: true,
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
		return nil, fmt.Errorf("codec %q not found", c.Codec)
	}

	client, err := resource.Get[dapr.Client](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return PublishAction(client, &c, codec), nil
}

func PublishAction(
	client dapr.Client,
	config *PublishConfig,
	codec codec.Codec) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var err error

		var input interface{} = data["input"]
		if config.Data != nil {
			input, err = config.Data.Eval(data)
			if err != nil {
				return nil, err
			}
		}

		var key string
		if config.Key != nil {
			key, err = expr.EvalAsStringE(config.Key, data)
			if err != nil {
				return nil, err
			}
		}

		var metadata map[string]string
		if config.Metadata != nil {
			if metadata, err = config.Metadata.EvalMap(data); err != nil {
				return nil, err
			}
		}

		// Propogate distributed tracing fields
		// per the the W3C TraceContext standard.
		if config.PropogateTracing {
			if m, ok := input.(map[string]interface{}); ok {
				otel.GetTextMapPropagator().Inject(ctx, tracing.MapCarrier(m))
			}
		}

		dataBytes, err := codec.Encode(input, config.CodecArgs...)
		if err != nil {
			return nil, err
		}

		if metadata == nil {
			metadata = make(map[string]string, 2)
		}
		if key != "" {
			metadata["partitionKey"] = key
		}
		metadata["rawPayload"] = "true"

		err = client.PublishEvent(ctx,
			config.Pubsub, config.Topic,
			dataBytes, dapr.PublishEventWithMetadata(metadata))

		return nil, resiliency.Retriable(err)
	}
}
