/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package dapr_pluggable

// import (
// 	"context"

// 	proto "github.com/dapr/dapr/pkg/proto/components/v1"

// 	"github.com/nanobus/nanobus/pkg/config"
// 	"github.com/nanobus/nanobus/pkg/resolve"
// 	"github.com/nanobus/nanobus/pkg/resource"
// )

// // Connection is the NamedLoader for a postgres connection.
// func PubSub() (string, resource.Loader) {
// 	return "dapr/pubsub.pluggable.v1", PubSubLoader
// }

// func PubSubLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
// 	var c ComponentConfig
// 	if err := config.Decode(with, &c); err != nil {
// 		return nil, err
// 	}

// 	conn, err := DialConfig(ctx, &c)
// 	if err != nil {
// 		return nil, err
// 	}

// 	client := proto.NewPubSubClient(conn)
// 	_, err = client.Init(ctx, &proto.PubSubInitRequest{
// 		Metadata: &proto.MetadataRequest{
// 			Properties: c.Properties,
// 		},
// 	})
// 	if err != nil {
// 		return nil, err
// 	}

// 	return client, nil
// }
