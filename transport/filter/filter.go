package filter

import (
	"context"

	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[Filter]
	Loader      = registry.Loader[Filter]
	Registry    = registry.Registry[Filter]

	Filter func(ctx context.Context, header Header) (context.Context, error)

	Header interface {
		Get(name string) string
		Values(name string) []string
	}
)
