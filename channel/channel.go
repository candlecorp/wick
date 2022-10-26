package channel

import (
	"context"

	"github.com/nanobus/nanobus/channel/metadata"
)

type (
	Receiver struct {
		Namespace string
		Operation string
		EntityID  string
	}

	///////////////////////////////////////////////////////
	// Function handling

	// Register registers a function handler with the transport layer.
	Register func(namespace, operation string, handler Handler)
	// Handler is a function that handles the invocation of a named function.
	Handler func(ctx context.Context, payload []byte) ([]byte, error)

	RegisterStateful func(namespace, operation string, method StatefulHandler)
	StatefulHandler  func(ctx context.Context, id string, payload []byte) ([]byte, error)

	///////////////////////////////////////////////////////
	// Function invoking

	// Invoke calls a function over the transport layer.
	Invoke func(ctx context.Context, receiver Receiver, payload []byte) ([]byte, error)

	InvokeStream func(ctx context.Context, receiver Receiver) (Streamer, error)

	Streamer interface {
		SendMetadata(md metadata.MD, end ...bool) error
		SendData(data []byte, end ...bool) error
		Metadata() metadata.MD
		RecvData() ([]byte, error)
		Close() error
	}
)

// Codec is an interface that handles encoding and decoding payloads send to and
// received from functions.
type Codec interface {
	ContentType() string
	Encode(v interface{}) ([]byte, error)
	Decode(data []byte, v interface{}) error
}
