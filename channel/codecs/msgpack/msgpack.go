package msgpack

import (
	"github.com/vmihailenco/msgpack/v5"
)

type Codec struct{}

func New() *Codec {
	return &Codec{}
}

func (c *Codec) ContentType() string {
	return "application/msgpack"
}

func (c *Codec) Encode(v interface{}) ([]byte, error) {
	return msgpack.Marshal(v)
}

func (c *Codec) Decode(data []byte, v interface{}) error {
	return msgpack.Unmarshal(data, v)
}
