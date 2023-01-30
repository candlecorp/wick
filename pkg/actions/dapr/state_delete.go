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
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resiliency"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func DeleteStateLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := DeleteStateConfig{
		Resource: "dapr",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	client, err := resource.Get[dapr.Client](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return DeleteStateAction(client, &c), nil
}

func DeleteStateAction(
	client dapr.Client,
	config *DeleteStateConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		key, err := expr.EvalAsStringE(config.Key, data)
		if err != nil {
			return nil, err
		}

		stateOptions := dapr.StateOptions{
			Concurrency: dapr.StateConcurrency(config.Concurrency),
			Consistency: dapr.StateConsistency(config.Consistency),
		}

		if config.Etag != nil {
			etag, err := expr.EvalAsStringE(config.Etag, data)
			if err != nil {
				return nil, fmt.Errorf("could not evaluate etag: %w", err)
			}

			if err = client.DeleteStateWithETag(ctx,
				config.Store, key,
				&dapr.ETag{Value: etag}, nil, &stateOptions); err != nil {
				return nil, resiliency.Retriable(err)
			}
		} else {
			if err = client.DeleteState(ctx, config.Store, key, nil); err != nil {
				return nil, resiliency.Retriable(err)
			}
		}

		return nil, nil
	}
}
