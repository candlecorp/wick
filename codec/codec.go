package codec

import (
	"github.com/nanobus/nanobus/resolve"
)

type (
	// Codec is an interface that handles encoding and decoding payloads send to and
	// received from functions.
	Codec interface {
		ContentType() string
		Encode(v interface{}, args ...interface{}) ([]byte, error)
		Decode(data []byte, args ...interface{}) (interface{}, string, error)
	}

	NamedLoader func() (string, bool, Loader)
	Loader      func(with interface{}, resolver resolve.ResolveAs) (Codec, error)
	Loadable    struct {
		Loader Loader
		Auto   bool
	}
	Registry map[string]Loadable
	Codecs   map[string]Codec
)

func (r Registry) Register(loaders ...NamedLoader) {
	for _, l := range loaders {
		name, auto, loader := l()
		r[name] = Loadable{
			Loader: loader,
			Auto:   auto,
		}
	}
}
