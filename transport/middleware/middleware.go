package middleware

import (
	"net/http"

	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[Middleware]
	Loader      = registry.Loader[Middleware]
	Registry    = registry.Registry[Middleware]

	Middleware func(h http.Handler) http.Handler
)
