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

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func SetStateLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := SetStateConfig{
		Resource: "dapr",
		Codec:    "json",
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

	return SetStateAction(client, codec, &c), nil
}

func SetStateAction(
	client dapr.Client,
	codec codec.Codec,
	config *SetStateConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		_r := [25]*dapr.SetStateItem{}
		r := _r[0:0]

		for i := range config.Items {
			configItems := &config.Items[i]
			var items []any
			if configItems.ForEach != nil {
				itemsInt, err := configItems.ForEach.Eval(data)
				if err != nil {
					return nil, fmt.Errorf("could not evaluate data: %w", err)
				}
				var ok bool
				if items, ok = itemsInt.([]any); !ok {
					return nil, fmt.Errorf("forEach expression %q did not return a slice of items", configItems.ForEach.Expr())
				}
			}

			if items == nil {
				it, err := createSetItem(data, nil, configItems, codec, config.CodecArgs)
				if err != nil {
					return nil, err
				}

				r = append(r, it)
			} else {
				for _, item := range items {
					it, err := createSetItem(data, item, configItems, codec, config.CodecArgs)
					if err != nil {
						return nil, err
					}

					r = append(r, it)
				}
			}
		}

		err := client.SaveBulkState(ctx, config.Store, r...)

		return nil, resiliency.Retriable(err)
	}
}

func createSetItem(
	data actions.Data,
	item any,
	config *SetStateItem,
	codec codec.Codec, args []any) (it *dapr.SetStateItem, err error) {
	variables := make(map[string]any, len(data)+1)
	for k, v := range data {
		variables[k] = v
	}
	variables["item"] = item

	var value any = variables["input"]
	it = &dapr.SetStateItem{}
	keyInt, err := config.Key.Eval(variables)
	if err != nil {
		return it, fmt.Errorf("could not evaluate key: %w", err)
	}
	it.Key = fmt.Sprintf("%v", keyInt)

	if config.Value != nil {
		if value, err = config.Value.Eval(variables); err != nil {
			return it, fmt.Errorf("could not evaluate value: %w", err)
		}
	}
	if config.Metadata != nil {
		if it.Metadata, err = config.Metadata.EvalMap(variables); err != nil {
			return it, fmt.Errorf("could not evaluate metadata: %w", err)
		}
	}
	if config.Etag != nil {
		etagInt, err := config.Etag.Eval(variables)
		if err != nil {
			return nil, fmt.Errorf("could not evaluate etag: %w", err)
		}
		it.Etag = &dapr.ETag{
			Value: fmt.Sprintf("%v", etagInt),
		}
	}

	jsonBytes, err := codec.Encode(value, args...)
	if err != nil {
		return it, fmt.Errorf("could not serialize value: %w", err)
	}
	it.Value = jsonBytes

	return it, nil
}
