/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package dapr_pluggable

import (
	"context"
	"fmt"

	proto "github.com/dapr/dapr/pkg/proto/components/v1"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type DeleteStateConfig struct {
	// Name is name of binding to invoke.
	Resource string `mapstructure:"resource" validate:"required"`
	// Operation is the name of the operation type for the binding to invoke.
	Key *expr.ValueExpr `mapstructure:"key" validate:"required"`
}

// DeleteState is the NamedLoader for the Dapr get state operation
func DeleteState() (string, actions.Loader) {
	return "@dapr_pluggable/delete_state", DeleteStateLoader
}

func DeleteStateLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := DeleteStateConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	client, err := resource.Get[proto.StateStoreClient](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return DeleteStateAction(client, &c), nil
}

func DeleteStateAction(
	client proto.StateStoreClient,
	config *DeleteStateConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		keyInt, err := config.Key.Eval(data)
		if err != nil {
			return nil, err
		}
		key := fmt.Sprintf("%v", keyInt)

		_, err = client.Delete(ctx, &proto.DeleteRequest{
			Key: key,
		})
		if err != nil {
			return nil, err
		}

		return nil, err
	}
}
