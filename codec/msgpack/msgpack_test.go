package msgpack_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/codec/msgpack"
)

func TestCodec(t *testing.T) {
	name, auto, loader := msgpack.MsgPack()
	assert.Equal(t, "msgpack", name)
	assert.True(t, auto)
	c, err := loader(nil, nil)
	require.NoError(t, err)
	assert.Equal(t, "application/msgpack", c.ContentType())
	data := map[string]interface{}{
		"int":    int64(1234),
		"string": "1234",
	}
	encoded, err := c.Encode(data)
	require.NoError(t, err)
	_, _, err = c.Decode([]byte{})
	assert.Error(t, err)
	decoded, _, err := c.Decode(encoded)
	require.NoError(t, err)
	assert.Equal(t, data, decoded)
}
