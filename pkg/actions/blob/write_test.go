package blob_test

import (
	"context"
	"io"
	"reflect"
	"testing"

	"github.com/go-logr/logr"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/blob/memblob"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/blob"
	"github.com/nanobus/nanobus/pkg/channel/metadata"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/codec/json"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/stream"
)

func TestWriteSingle(t *testing.T) {
	ctx := context.Background()
	logger := logr.Discard()

	m := memblob.OpenBucket(&memblob.Options{})
	resources := resource.Resources{
		"test": m,
	}
	jsonCodec := json.NewCodec()
	codecs := codec.Codecs{
		"json": jsonCodec,
	}

	resolver := func(name string, target interface{}) bool {
		switch name {
		case "system:logger":
			return resolve.As(logger, target)
		case "resource:lookup":
			return resolve.As(resources, target)
		case "codec:lookup":
			return resolve.As(codecs, target)
		}
		return false
	}

	expected := map[string]any{
		"foo": "bar",
	}
	data := actions.Data{
		"input": map[string]any{
			"key":     "1234",
			"content": expected,
		},
	}

	a, err := blob.WriteLoader(ctx, map[string]any{
		"resource": "test",
		"key":      "input.key",
		"data":     "input.content",
		"codec":    "json",
	}, resolver)
	require.NoError(t, err)

	_, err = a(ctx, data)
	require.NoError(t, err)

	keyBytes, err := m.ReadAll(ctx, "1234")
	require.NoError(t, err)

	actual, _, err := jsonCodec.Decode(keyBytes)
	require.NoError(t, err)

	assert.Equal(t, expected, actual)
}

func TestWriteStream(t *testing.T) {
	ctx := context.Background()
	logger := logr.Discard()

	ch := make(chan any, 1000)
	mstr := &mockSource{
		ch: ch,
	}
	ctx = stream.SourceNewContext(ctx, mstr)

	m := memblob.OpenBucket(&memblob.Options{})
	resources := resource.Resources{
		"test": m,
	}
	jsonCodec := json.NewCodec()
	codecs := codec.Codecs{
		"json": jsonCodec,
	}

	resolver := func(name string, target interface{}) bool {
		switch name {
		case "system:logger":
			return resolve.As(logger, target)
		case "resource:lookup":
			return resolve.As(resources, target)
		case "codec:lookup":
			return resolve.As(codecs, target)
		}
		return false
	}

	expected := map[string]any{
		"foo": "bar",
	}
	data := actions.Data{
		"input": map[string]any{
			"key": "1234",
		},
	}

	ch <- expected
	close(ch)

	a, err := blob.WriteLoader(ctx, map[string]any{
		"resource": "test",
		"key":      "input.key",
		"codec":    "json",
	}, resolver)
	require.NoError(t, err)

	_, err = a(ctx, data)
	require.NoError(t, err)

	keyBytes, err := m.ReadAll(ctx, "1234")
	require.NoError(t, err)

	actual, _, err := jsonCodec.Decode(keyBytes)
	require.NoError(t, err)

	assert.Equal(t, expected, actual)
}

type mockSource struct {
	stream.Source
	ch  chan any
	err error
}

func (m *mockSource) Next(data any, md *metadata.MD) error {
	if m.err != nil {
		return m.err
	}

	source, ok := <-m.ch
	if !ok {
		return io.EOF
	}

	val := reflect.ValueOf(data)
	typ := val.Type()
	if typ.Kind() != reflect.Ptr || val.IsNil() {
		return nil
	}

	targetType := typ.Elem()
	if reflect.TypeOf(source).AssignableTo(targetType) {
		val.Elem().Set(reflect.ValueOf(source))
		return nil
	}

	return nil
}

func (m *mockSource) Cancel() {
}
