package blob_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/blob/memblob"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/blob"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/codec/json"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func TestReadSingle(t *testing.T) {
	ctx := context.Background()

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
			"key":     "1234",
			"content": expected,
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
