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
// 	"fmt"
// 	"path/filepath"

// 	"google.golang.org/grpc"
// 	"google.golang.org/grpc/credentials/insecure"
// )

// type ComponentConfig struct {
// 	SocketPath string            `mapstructure:"socketPath" required_without:"address"`
// 	Address    string            `mapstructure:"address" required_without:"socketPath"`
// 	Properties map[string]string `mapstructure:"properties" validate:"required"`
// }

// func DialSocketContext(ctx context.Context, filename string, additionalOpts ...grpc.DialOption) (*grpc.ClientConn, error) {
// 	absPath, err := filepath.Abs(filename)
// 	if err != nil {
// 		return nil, err
// 	}
// 	address := "unix://" + absPath
// 	return DialContext(ctx, address, additionalOpts...)
// }

// func DialContext(ctx context.Context, address string, additionalOpts ...grpc.DialOption) (*grpc.ClientConn, error) {
// 	additionalOpts = append(additionalOpts, grpc.WithTransportCredentials(insecure.NewCredentials()))

// 	grpcConn, err := grpc.DialContext(ctx, address, additionalOpts...)
// 	if err != nil {
// 		return nil, fmt.Errorf("unable to open GRPC connection to address %q: %w", address, err)
// 	}

// 	return grpcConn, nil
// }

// func DialConfig(ctx context.Context, config *ComponentConfig) (*grpc.ClientConn, error) {
// 	if config.SocketPath != "" {
// 		return DialSocketContext(ctx, config.SocketPath)
// 	}
// 	return DialContext(ctx, config.Address)
// }
