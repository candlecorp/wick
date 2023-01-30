package blob_test

import (
	"bytes"
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/blob/memblob"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/blob"
	"github.com/nanobus/nanobus/pkg/channel/metadata"
	"github.com/nanobus/nanobus/pkg/codec"
	json_codec "github.com/nanobus/nanobus/pkg/codec/json"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/stream"
)

func TestReadSingle(t *testing.T) {
	ctx := context.Background()

	m := memblob.OpenBucket(&memblob.Options{})
	resources := resource.Resources{
		"test": m,
	}
	jsonCodec := json_codec.NewCodec()
	codecs := codec.Codecs{
		"json": jsonCodec,
	}

	resolver := func(name string, target interface{}) bool {
		switch name {
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

	encoded, err := jsonCodec.Encode(expected)
	require.NoError(t, err)
	require.NoError(t, m.WriteAll(ctx, "1234", encoded, nil))

	data := actions.Data{
		"input": map[string]any{
			"key": "1234",
		},
	}

	a, err := blob.ReadLoader(ctx, map[string]any{
		"resource": "test",
		"key":      "input.key",
		"codec":    "json",
	}, resolver)
	require.NoError(t, err)

	actual, err := a(ctx, data)
	require.NoError(t, err)

	assert.Equal(t, expected, actual)
}

func TestReadStream(t *testing.T) {
	ctx := context.Background()

	m := memblob.OpenBucket(&memblob.Options{})
	resources := resource.Resources{
		"test": m,
	}
	jsonCodec := json_codec.NewCodec()
	codecs := codec.Codecs{
		"json": jsonCodec,
	}

	resolver := func(name string, target interface{}) bool {
		switch name {
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

	encoded, err := jsonCodec.Encode(expected)
	require.NoError(t, err)
	require.NoError(t, m.WriteAll(ctx, "1234", encoded, nil))

	data := actions.Data{
		"input": map[string]any{
			"key": "1234",
		},
	}

	s := &mockSink{
		data: make([]any, 0, 1024),
	}
	ctx = stream.SinkNewContext(ctx, s)

	a, err := blob.ReadLoader(ctx, map[string]any{
		"resource":   "test",
		"key":        "input.key",
		"codec":      "json",
		"bufferSize": 10,
	}, resolver)
	require.NoError(t, err)

	_, err = a(ctx, data)
	require.NoError(t, err)

	buf := bytes.Buffer{}
	for _, data := range s.data {
		b := data.([]byte)
		fmt.Println("Got BYTES", b)
		n, err := buf.Write(b)
		fmt.Println(n, err)
	}

	s.Complete()

	fmt.Println("TEST 1")
	actual := buf.Bytes()
	fmt.Println("TEST 2", actual)

	assert.Equal(t, encoded, actual)
}

type mockSink struct {
	stream.Sink
	data     []any
	complete bool
	err      error
}

func (m *mockSink) Next(data any, md metadata.MD) error {
	m.data = append(m.data, data)
	return nil
}

func (m *mockSink) Complete() {
	m.complete = true
}
func (m *mockSink) Error(err error) {
	m.err = err
}
