/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package wapc

// import (
// 	"context"
// 	"errors"
// 	"fmt"
// 	"os"
// 	"runtime"
// 	"strings"

// 	"github.com/nanobus/nanobus/pkg/channel"
// 	wapc_mux "github.com/nanobus/nanobus/pkg/channel/transports/wapc"
// 	wapc "github.com/wapc/wapc-go"
// 	"github.com/wapc/wapc-go/engines/wazero"

// 	"github.com/nanobus/nanobus/pkg/compute"
// 	"github.com/nanobus/nanobus/pkg/config"
// 	"github.com/nanobus/nanobus/pkg/errorz"
// 	"github.com/nanobus/nanobus/pkg/resolve"
// )

// type WaPCConfig struct {
// 	// Filename is the file name of the waPC/WebAssembly module to load.
// 	Filename runtime.FilePath `mapstructure:"filename" validate:"required"` // TODO: Load from external location
// 	// PoolSize controls the number of waPC instance of the module to create and pool.
// 	// It also represents the maximum number of concurrent requests the module can process.
// 	PoolSize uint64 `mapstructure:"poolSize"`
// }

// // Mux is the NamedLoader for the waPC compute.
// func WaPC() (string, compute.Loader) {
// 	return "wapc", WaPCLoader
// }

// func WaPCLoader(with interface{}, resolver resolve.ResolveAs) (*compute.Compute, error) {
// 	var busInvoker compute.BusInvoker
// 	var msgpackcodec channel.Codec
// 	if err := resolve.Resolve(resolver,
// 		"bus:invoker", &busInvoker,
// 		"codec:msgpack", &msgpackcodec); err != nil {
// 		return nil, err
// 	}

// 	c := WaPCConfig{
// 		PoolSize: uint64(runtime.NumCPU() * 5),
// 	}
// 	if err := config.Decode(with, &c); err != nil {
// 		return nil, err
// 	}

// 	wasmBytes, err := os.ReadFile(c.Filename)
// 	if err != nil {
// 		return nil, err
// 	}

// 	engine := wazero.Engine()

// 	module, err := engine.New(context.Background(), func(ctx context.Context, binding, namespace, operation string, payload []byte) ([]byte, error) {
// 		lastDot := strings.LastIndexByte(namespace, '.')
// 		if lastDot < 0 {
// 			return nil, fmt.Errorf("invalid namespace %q", namespace)
// 		}
// 		service := namespace[lastDot+1:]
// 		namespace = namespace[:lastDot]

// 		var input interface{}
// 		if err := msgpackcodec.Decode(payload, &input); err != nil {
// 			return nil, err
// 		}

// 		result, err := busInvoker(ctx, namespace, service, operation, input)
// 		if err != nil {
// 			return nil, err
// 		}

// 		return msgpackcodec.Encode(result)
// 	}, wasmBytes, &wapc.ModuleConfig{
// 		Logger: wapc.PrintlnLogger,
// 		Stdout: os.Stdout,
// 		Stderr: os.Stderr,
// 	})
// 	if err != nil {
// 		return nil, err
// 	}

// 	m, err := wapc_mux.New(module, uint64(c.PoolSize))
// 	if err != nil {
// 		return nil, err
// 	}
// 	invoke := func(ctx context.Context, receiver channel.Receiver, payload []byte) ([]byte, error) {
// 		resp, err := m.Invoke(ctx, receiver, payload)
// 		if err != nil {
// 			// Trim out wrapped message.
// 			msg := err.Error()
// 			msg = strings.TrimPrefix(msg, "Host error: ")
// 			i := strings.Index(msg, "; ~lib/@wapc/")
// 			if i > 0 {
// 				msg = msg[:i]
// 			}
// 			return nil, errors.New(msg)
// 		}
// 		return resp, nil
// 	}
// 	invokeStream := func(ctx context.Context, receiver channel.Receiver) (channel.Streamer, error) {
// 		return nil, errorz.New(errorz.Unimplemented, "streaming is not implemented for waPC")
// 	}
// 	invoker := channel.NewInvoker(invoke, invokeStream, msgpackcodec)
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
