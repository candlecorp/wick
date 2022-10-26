package json

import (
	"encoding/json"
)

type Codec struct{}

func New() *Codec {
	return &Codec{}
}

func (c *Codec) ContentType() string {
	return "application/json"
}

func (c *Codec) Encode(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}

func (c *Codec) Decode(data []byte, v interface{}) error {
	return json.Unmarshal(data, v)
}
