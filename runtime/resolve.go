package runtime

import (
	"errors"
	"fmt"
	"reflect"
)

type DependencyResolver func(name string) (interface{}, bool)

type ResolveAs func(name string, target interface{}) bool

func Resolve(resolver ResolveAs, args ...interface{}) error {
	if len(args)%2 != 0 {
		return errors.New("invalid number of arguments passed to Resolve")
	}

	for i := 0; i < len(args); i += 2 {
		dependencyName, ok := args[i].(string)
		if !ok {
			return fmt.Errorf("argment %d is not a string", i)
		}

		if !resolver(dependencyName, args[i+1]) {
			return fmt.Errorf("could not resolve dependency %q", dependencyName)
		}
	}

	return nil
}

func ToResolveAs(resolver DependencyResolver) ResolveAs {
	return func(name string, target interface{}) bool {
		dependency, ok := resolver(name)
		if !ok {
			return false
		}

		return as(dependency, target)
	}
}

func as(source, target interface{}) bool {
	if target == nil {
		return false
	}
	val := reflect.ValueOf(target)
	typ := val.Type()
	if typ.Kind() != reflect.Ptr || val.IsNil() {
		return false
	}

	targetType := typ.Elem()
	if reflect.TypeOf(source).AssignableTo(targetType) {
		val.Elem().Set(reflect.ValueOf(source))
		return true
	}

	return false
}
