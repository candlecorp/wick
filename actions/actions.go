package actions

import (
	"context"
	"errors"

	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[Action]
	Loader      = registry.Loader[Action]
	Registry    = registry.Registry[Action]

	Data   map[string]interface{}
	Action func(ctx context.Context, data Data) (interface{}, error)
)

func (d Data) Clone() Data {
	clone := make(Data, len(d)+5)
	for k, v := range d {
		clone[k] = v
	}
	return clone
}

// ErrStop is returned by an action when the processing should stop.
var ErrStop = errors.New("processing stopped")

func Stop() error {
	return ErrStop
}
