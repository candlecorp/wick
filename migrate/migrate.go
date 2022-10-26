package migrate

import (
	"context"

	_ "github.com/golang-migrate/migrate/v4/source/file"
	_ "github.com/lib/pq"

	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[Migrater]
	Loader      = registry.Loader[Migrater]
	Registry    = registry.Registry[Migrater]

	Migrater func(ctx context.Context) error
)
