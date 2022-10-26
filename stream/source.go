package stream

import (
	"io"

	"github.com/nanobus/iota/go/wasmrs/payload"
	"github.com/nanobus/iota/go/wasmrs/rx/flux"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/nanobus/nanobus/channel/metadata"
)

type source struct {
	f  flux.Flux[payload.Payload]
	ch chan value
}

type value struct {
	p   payload.Payload
	err error
}

func SourceFromFlux(f flux.Flux[payload.Payload]) Source {
	ch := make(chan value, 100)
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
	})
	return &source{
		f:  f,
		ch: ch,
	}
}

func (s *source) Next(data any, md *metadata.MD) error {
	val, ok := <-s.ch
	if val.err != nil {
		return val.err
	}
	if ok && val.p != nil {
		return msgpack.Unmarshal(val.p.Data(), data)
	}

	return io.EOF
}
