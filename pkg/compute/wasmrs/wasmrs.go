/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package wasmrs

import (
	"context"
	"net/url"
	"os"

	"github.com/nanobus/iota/go/transport/wasmrs/host"

	"github.com/nanobus/nanobus/pkg/compute"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/runtime"
)

type Config struct {
	// Filename is the file name of the WasmRS module to load.
	// TODO: Load from external location
	Filename runtime.FilePath `mapstructure:"filename" validate:"required"`
}

// WasmRS
func WasmRS() (string, compute.Loader) {
	return "wasmrs", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (compute.Invoker, error) {
	c := Config{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	targetUrl, err := url.Parse(string(c.Filename))
	if err != nil {
		return nil, err
	}

	source, err := os.ReadFile(targetUrl.Path)
	if err != nil {
		return nil, err
	}

	h, err := host.New(ctx)
	if err != nil {
		return nil, err
	}
	module, err := h.Compile(ctx, source)
	if err != nil {
		return nil, err
	}

	return module.Instantiate(ctx)
}
