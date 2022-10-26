package jaeger

import (
	"context"

	"go.opentelemetry.io/otel/exporters/jaeger"
	"go.opentelemetry.io/otel/sdk/trace"

	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/telemetry/tracing"
)

type Config struct {
	// CollectorEndpoint is the endpoint for jaeger span collection.
	CollectorEndpoint string `mapstructure:"collectorEndpoint"`
	Username          string `mapstructure:"username"`
	Password          string `mapstructure:"password"`
}

// Jaeger is the NamedLoader for Jaeger.
func Jaeger() (string, tracing.Loader) {
	return "jaeger", Loader
}

func Loader(ctx context.Context, with interface{}, resolveAs resolve.ResolveAs) (trace.SpanExporter, error) {
	c := Config{
		CollectorEndpoint: "http://localhost:14268/api/traces",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	opts := []jaeger.CollectorEndpointOption{
		jaeger.WithEndpoint(c.CollectorEndpoint),
	}

	if c.Username != "" {
		opts = append(opts, jaeger.WithUsername(c.Username))
	}
	if c.Password != "" {
		opts = append(opts, jaeger.WithPassword(c.Password))
	}

	return jaeger.New(jaeger.WithCollectorEndpoint(opts...))
}
