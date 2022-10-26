package resource

import (
	"fmt"
	"reflect"

	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[any]
	Loader      = registry.Loader[any]
	Registry    = registry.Registry[any]

	// NamedLoader func() (string, Loader)
	// Loader      func(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error)
	// Registry    map[string]Loader
	Resources map[string]interface{}
)

// func (r Registry) Register(loaders ...NamedLoader) {
// 	for _, l := range loaders {
// 		name, loader := l()
// 		r[name] = loader
// 	}
// }

func Get[T any](r Resources, name string) (res T, err error) {
	var iface interface{}
	iface, ok := r[name]
	if !ok {
		return res, fmt.Errorf("resource %q is not registered", name)
	}
	res, ok = iface.(T)
	if !ok {
		return res, fmt.Errorf("resource %q is not a %s", name, reflect.TypeOf(res).Name())
	}

	return res, nil
}
