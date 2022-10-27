package routes

import (
	"github.com/gorilla/mux"
	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[AddRoutes]
	Loader      = registry.Loader[AddRoutes]
	Registry    = registry.Registry[AddRoutes]

	AddRoutes func(r *mux.Router)
)
