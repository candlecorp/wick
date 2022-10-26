package registry

import (
	"context"

	"github.com/nanobus/nanobus/resolve"
)

type (
	NamedLoader[T any] func() (string, Loader[T])
	Loader[T any]      func(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (T, error)
	Registry[T any]    map[string]Loader[T]
)

func (r Registry[T]) Register(loaders ...NamedLoader[T]) {
	for _, l := range loaders {
		name, loader := l()
		r[name] = loader
	}
}
