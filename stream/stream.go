package stream

import (
	"context"

	"github.com/nanobus/nanobus/channel/metadata"
)

type Source interface {
	Next(data any, md *metadata.MD) error
}

type sourceKey struct{}

// SourceNewContext creates a new context with incoming `s` attached.
func SourceNewContext(ctx context.Context, s Source) context.Context {
	return context.WithValue(ctx, sourceKey{}, s)
}

func SourceFromContext(ctx context.Context) (s Source, ok bool) {
	s, ok = ctx.Value(sourceKey{}).(Source)
	return
}

type Sink interface {
	Next(data any, md metadata.MD) error
	Complete()
	Error(err error)
}

type sinkKey struct{}

// SinkNewContext creates a new context with incoming `s` attached.
func SinkNewContext(ctx context.Context, s Sink) context.Context {
	return context.WithValue(ctx, sinkKey{}, s)
}

func SinkFromContext(ctx context.Context) (s Sink, ok bool) {
	s, ok = ctx.Value(sinkKey{}).(Sink)
	return
}
