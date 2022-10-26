package transport

import (
	"context"
	"errors"

	"github.com/nanobus/nanobus/resolve"
)

var ErrBadInput = errors.New("input was malformed")

type (
	NamedLoader func() (string, Loader)
	Loader      func(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (Transport, error)

	Transport interface {
		Listen() error
		Close() error
	}

	Invoker func(ctx context.Context, namespace, service, id, function string, input interface{}) (interface{}, error)

	Registry map[string]Loader
)

func (r Registry) Register(loaders ...NamedLoader) {
	for _, l := range loaders {
		name, loader := l()
		r[name] = loader
	}
}
