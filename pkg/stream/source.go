/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package stream

import (
	"io"
	"sync/atomic"

	"github.com/nanobus/iota/go/payload"
	"github.com/nanobus/iota/go/rx"
	"github.com/nanobus/iota/go/rx/flux"

	"github.com/nanobus/nanobus/pkg/channel"
	"github.com/nanobus/nanobus/pkg/channel/metadata"
)

type source struct {
	codec    channel.Codec
	f        flux.Flux[payload.Payload]
	ch       chan value
	canceled atomic.Bool
}

type value struct {
	p   payload.Payload
	err error
}

func SourceFromFlux(codec channel.Codec, f flux.Flux[payload.Payload]) Source {
	ch := make(chan value, 100)
	s := &source{
		codec: codec,
		f:     f,
		ch:    ch,
	}
	f.Subscribe(flux.Subscribe[payload.Payload]{
		OnNext: func(p payload.Payload) {
			ch <- value{p: p}
		},
		OnComplete: func() {
			close(ch)
		},
		OnError: func(err error) {
			ch <- value{err: err}
			close(ch)
		},
		OnRequest: func(sub rx.Subscription) {
			if s.canceled.Load() {
				sub.Cancel()
			} else {
				sub.Request(100)
			}
		},
	})
	return s
}

func (s *source) Next(data any, md *metadata.MD) error {
	val, ok := <-s.ch
	if val.err != nil {
		return val.err
	}
	if ok && val.p != nil {
		return s.codec.Decode(val.p.Data(), data)
	}

	return io.EOF
}

func (s *source) Cancel() {
	s.canceled.Store(true)
}
