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

	dapr "github.com/dapr/go-sdk/client"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type ClientConfig struct {
	Port    *string `mapstructure:"port"`
	Address *string `mapstructure:"address"`
	Socket  *string `mapstructure:"socket"`
}

// Connection is the NamedLoader for a postgres connection.
func Client() (string, resource.Loader) {
	return "nanobus.resource.dapr/v1", ClientLoader
}

func ClientLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c ClientConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var client dapr.Client
	var err error

	if c.Port != nil {
		client, err = dapr.NewClientWithPort(*c.Port)
	} else if c.Address != nil {
		client, err = dapr.NewClientWithAddress(*c.Address)
	} else if c.Socket != nil {
		client, err = dapr.NewClientWithSocket(*c.Socket)
	} else {
		client, err = dapr.NewClient()
	}
	if err != nil {
		return nil, err
	}

	return client, nil
}
