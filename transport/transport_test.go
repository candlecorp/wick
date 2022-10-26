package transport_test

import (
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/transport"
)

func TestRegistry(t *testing.T) {
	r := transport.Registry{}

	loader := func(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (transport.Transport, error) {
		return nil, nil
	}
	namedLoader := func() (string, transport.Loader) {
		return "test", loader
	}

	r.Register(namedLoader)

	assert.Equal(t, fmt.Sprintf("%v", transport.Loader(loader)), fmt.Sprintf("%p", r["test"]))
}
