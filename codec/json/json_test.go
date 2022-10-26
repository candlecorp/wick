package json_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/codec/json"
)

func TestCodec(t *testing.T) {
	name, auto, loader := json.JSON()
	assert.Equal(t, "json", name)
	assert.True(t, auto)
	c, err := loader(nil, nil)
	require.NoError(t, err)
	assert.Equal(t, "application/json", c.ContentType())
	data := map[string]interface{}{
		"int":    int64(1234),
		"string": "1234",
	}
	encoded, err := c.Encode(data)
	require.NoError(t, err)
	_, _, err = c.Decode([]byte(`bad data`))
	assert.Error(t, err)
	decoded, _, err := c.Decode(encoded)
	require.NoError(t, err)
	assert.Equal(t, data, decoded)
}
