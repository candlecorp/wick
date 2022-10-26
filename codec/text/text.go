package text

import (
	"fmt"

	"github.com/nanobus/nanobus/codec"
	"github.com/nanobus/nanobus/resolve"
)

type (
	// Codec encodes and decodes Avro records.
	Codec struct {
		contentType string
	}
)

// Plain is the NamedLoader for the `text/plain` codec.
func Plain() (string, bool, codec.Loader) {
	return "text/plain", true, func(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
		return NewCodec("text/plain"), nil
	}
}

// HTML is the NamedLoader for the `text/html` codec.
func HTML() (string, bool, codec.Loader) {
	return "text/html", true, func(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
		return NewCodec("text/html"), nil
	}
}

// NewCodec creates a `Codec`.
func NewCodec(contentType string) *Codec {
	return &Codec{
		contentType: contentType,
	}
}

func (c *Codec) ContentType() string {
	return c.contentType
}

// Decode decodes JSON bytes to a value.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	return string(msgValue), "", nil
}

// Encode encodes a value into JSON encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	msg := fmt.Sprintf("%v", value)
	return []byte(msg), nil
}
