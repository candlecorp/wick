package stdout

import (
	"context"
	"os"
	"os/signal"
	"syscall"

	"go.opentelemetry.io/otel/exporters/stdout/stdouttrace"
	"go.opentelemetry.io/otel/sdk/trace"

	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/runtime"
	"github.com/nanobus/nanobus/telemetry/tracing"
)

type Config struct {
	Filename          runtime.FilePath `mapstructure:"filename"`
	PrettyPrint       bool             `mapstructure:"prettyPrint"`
	WithoutTimestamps bool             `mapstructure:"withoutTimestamps"`
}

// Jaeger is the NamedLoader for Jaeger.
func Stdout() (string, tracing.Loader) {
	return "stdout", Loader
}

func Loader(ctx context.Context, with interface{}, resolveAs resolve.ResolveAs) (trace.SpanExporter, error) {
	c := Config{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}
	opts := []stdouttrace.Option{}

	// Write telemetry data to a file.
	if c.Filename != "" {
		f, err := os.Create(string(c.Filename))
		if err != nil {
			return nil, err
		}

		opts = append(opts, stdouttrace.WithWriter(f))

		go func() {
			s := make(chan os.Signal, 1)

			// add any other syscalls that you want to be notified with
			signal.Notify(s, syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
			<-s

			f.Close()
		}()
	}

	// Use human-readable output.
	if c.PrettyPrint {
		opts = append(opts, stdouttrace.WithPrettyPrint())
	}
	// Do not print timestamps for the demo.
	if c.WithoutTimestamps {
		opts = append(opts, stdouttrace.WithoutTimestamps())
	}

	return stdouttrace.New(opts...)
}
