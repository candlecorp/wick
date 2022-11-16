/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package stream

import (
	"github.com/nanobus/iota/go/wasmrs/payload"
	"github.com/nanobus/iota/go/wasmrs/rx/flux"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/nanobus/nanobus/pkg/channel/metadata"
)

type sink struct {
	f flux.Sink[payload.Payload]
}

func FromSink(f flux.Sink[payload.Payload]) Sink {
	return &sink{
		f: f,
	}
}

func (s *sink) Next(data any, md metadata.MD) error {
	dataBytes, err := msgpack.Marshal(data)
	if err != nil {
		return err
	}
	s.f.Next(payload.New(dataBytes))
	return nil
}

func (s *sink) Complete() {
	s.f.Complete()
}

func (s *sink) Error(err error) {
	s.f.Error(err)
}
