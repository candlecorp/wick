package tracing

import (
	"go.opentelemetry.io/otel/sdk/trace"

	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[trace.SpanExporter]
	Loader      = registry.Loader[trace.SpanExporter]
	Registry    = registry.Registry[trace.SpanExporter]
)
