package otlp

import (
	"context"
	"fmt"
	"strings"
	"time"

	"go.opentelemetry.io/otel/exporters/otlp/otlptrace"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp"
	"go.opentelemetry.io/otel/sdk/trace"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/telemetry/tracing"
)

type Config struct {
	Protocol string `mapstructure:"protocol"`
	Address  string `mapstructure:"address"`

	Endpoint    string        `mapstructure:"endpoint"`
	Compression string        `mapstructure:"compression"`
	URLPath     string        `mapstructure:"urlPath"`
	Insecure    bool          `mapstructure:"insecure"`
	Timeout     time.Duration `mapstructure:"timeout"`
	Retry       *Retry        `mapstructure:"retry"`
}

type Retry struct {
	// Enabled indicates whether to not retry sending batches in case of
	// export failure.
	Enabled bool `mapstructure:"enabled"`
	// InitialInterval the time to wait after the first failure before
	// retrying.
	InitialInterval time.Duration `mapstructure:"initialInterval"`
	// MaxInterval is the upper bound on backoff interval. Once this value is
	// reached the delay between consecutive retries will always be
	// `MaxInterval`.
	MaxInterval time.Duration `mapstructure:"maxInterval"`
	// MaxElapsedTime is the maximum amount of time (including retries) spent
	// trying to send a request/batch.  Once this value is reached, the data
	// is discarded.
	MaxElapsedTime time.Duration `mapstructure:"maxElapsedTime"`
}

// OTLP is the NamedLoader for OTLP.
func OTLP() (string, tracing.Loader) {
	return "otlp", Loader
}

func Loader(ctx context.Context, with interface{}, resolveAs resolve.ResolveAs) (trace.SpanExporter, error) {
	c := Config{
		Protocol: "grpc",
		Address:  "localhost:30080",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	switch strings.ToLower(c.Protocol) {
	case "grpc":
		ctx, cancel := context.WithTimeout(ctx, time.Second)
		defer cancel()
		conn, err := grpc.DialContext(ctx, c.Address,
			grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
		if err != nil {
			return nil, fmt.Errorf("failed to create gRPC connection to collector: %w", err)
		}

		// Set up a trace exporter
		opts := []otlptracegrpc.Option{
			otlptracegrpc.WithGRPCConn(conn),
		}
		if c.Compression != "" {
			opts = append(opts, otlptracegrpc.WithCompressor(c.Compression))
		}
		if c.Endpoint != "" {
			opts = append(opts, otlptracegrpc.WithEndpoint(c.Endpoint))
		}
		if c.Timeout != 0 {
			opts = append(opts, otlptracegrpc.WithTimeout(c.Timeout))
		}
		if c.Retry != nil {
			opts = append(opts, otlptracegrpc.WithRetry(otlptracegrpc.RetryConfig{
				Enabled:         c.Retry.Enabled,
				InitialInterval: c.Retry.InitialInterval,
				MaxInterval:     c.Retry.MaxInterval,
				MaxElapsedTime:  c.Retry.MaxInterval,
			}))
		}

		return otlptracegrpc.New(ctx, opts...)
	case "http":
		opts := []otlptracehttp.Option{}
		if c.Endpoint != "" {
			opts = append(opts, otlptracehttp.WithEndpoint(c.Endpoint))
		}
		if c.Compression == "gzip" {
			opts = append(opts, otlptracehttp.WithCompression(otlptracehttp.GzipCompression))
		}
		if c.URLPath != "" {
			opts = append(opts, otlptracehttp.WithURLPath(c.URLPath))
		}
		if c.Insecure {
			opts = append(opts, otlptracehttp.WithInsecure())
		}
		if c.Timeout != 0 {
			opts = append(opts, otlptracehttp.WithTimeout(c.Timeout))
		}
		if c.Retry != nil {
			opts = append(opts, otlptracehttp.WithRetry(otlptracehttp.RetryConfig{
				Enabled:         c.Retry.Enabled,
				InitialInterval: c.Retry.InitialInterval,
				MaxInterval:     c.Retry.MaxInterval,
				MaxElapsedTime:  c.Retry.MaxInterval,
			}))
		}
		client := otlptracehttp.NewClient(opts...)
		exporter, err := otlptrace.New(ctx, client)
		if err != nil {
			return nil, fmt.Errorf("creating OTLP trace exporter: %w", err)
		}
		return exporter, nil
	}

	return nil, fmt.Errorf("unexpected protocol %s", c.Protocol)
}
