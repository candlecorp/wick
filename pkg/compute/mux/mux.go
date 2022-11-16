/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package mux

// import (
// 	"context"
// 	"errors"
// 	"os"

// 	"github.com/nanobus/nanobus/pkg/channel"
// 	msgpack_codec "github.com/nanobus/nanobus/pkg/channel/codecs/msgpack"
// 	transport_mux "github.com/nanobus/nanobus/pkg/channel/transports/mux"

// 	"github.com/nanobus/nanobus/pkg/compute"
// 	"github.com/nanobus/nanobus/pkg/config"
// 	"github.com/nanobus/nanobus/pkg/errorz"
// 	"github.com/nanobus/nanobus/pkg/resolve"
// )

// const defaultInvokeURL = "http://127.0.0.1:9000"

// type MuxConfig struct {
// 	BaseURL string `mapstructure:"baseUrl"`
// }

// // Mux is the NamedLoader for the mux compute.
// func Mux() (string, compute.Loader) {
// 	return "mux", MuxLoader
// }

// func MuxLoader(with interface{}, resolver resolve.ResolveAs) (*compute.Compute, error) {
// 	baseURL := os.Getenv("APP_URL")
// 	if baseURL == "" {
// 		baseURL = defaultInvokeURL
// 	}
// 	c := MuxConfig{
// 		BaseURL: baseURL,
// 	}
// 	if err := config.Decode(with, &c); err != nil {
// 		return nil, err
// 	}

// 	msgpackcodec := msgpack_codec.New()
// 	m := transport_mux.New(c.BaseURL, msgpackcodec.ContentType())
// 	invokeStream := func(ctx context.Context, receiver channel.Receiver) (channel.Streamer, error) {
// 		return nil, errors.New(errorz.Unimplemented.String())
// 	}
// 	invoker := channel.NewInvoker(m.Invoke, invokeStream, msgpackcodec)
// 	done := make(chan struct{}, 1)

// 	return &compute.Compute{
// 		Invoker: invoker,
// 		Start:   func() error { return nil },
// 		WaitUntilShutdown: func() error {
// 			<-done
// 			return nil
// 		},
// 		Close: func() error {
// 			close(done)
// 			return nil
// 		},
// 	}, nil
// }
