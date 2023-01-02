/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package wapc

import (
	"context"
	"runtime"

	wapc "github.com/wapc/wapc-go"

	functions "github.com/nanobus/nanobus/pkg/channel"
	"github.com/nanobus/nanobus/pkg/logger"
)

const DefaultPoolSize = 0

type WaPC struct {
	module wapc.Module
	pool   *wapc.Pool
}

// Ensure `Invoke` conforms to `functions.Invoke`
var _ = (functions.Invoke)(((*WaPC)(nil)).Invoke)

// Registering handlers is handled by waPC itself.

func New(module wapc.Module, poolSize uint64) (*WaPC, error) {
	if poolSize == DefaultPoolSize {
		poolSize = uint64(runtime.NumCPU() * 2)
	}
	pool, err := wapc.NewPool(context.Background(), module, poolSize)
	if err != nil {
		return nil, err
	}

	return &WaPC{
		module: module,
		pool:   pool,
	}, nil
}

func (w *WaPC) Invoke(ctx context.Context, receiver functions.Receiver, payload []byte) ([]byte, error) {
	instance, err := w.pool.Get(0)
	if err != nil {
		return nil, err
	}
	defer func() {
		err := w.pool.Return(instance)
		if err != nil {
			logger.Error("could not return WebAssembly module back to the pool", "err", err)
		}
	}()

	return instance.Invoke(ctx, receiver.Namespace+"/"+receiver.Operation, payload)
}
