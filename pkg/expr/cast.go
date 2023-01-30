package expr

import (
	"fmt"
	"reflect"
	"strconv"

	"github.com/spf13/cast"
)

type Expr interface {
	Eval(variables map[string]interface{}) (interface{}, error)
}

func EvalAsInt64E(e Expr, data map[string]interface{}) (int64, error) {
	ival, err := e.Eval(data)
	if err != nil {
		return 0, err
	}
	return cast.ToInt64E(ival)
}

func EvalAsStringE(e Expr, data map[string]interface{}) (string, error) {
	ival, err := e.Eval(data)
	if err != nil {
		return "", err
	}
	return cast.ToStringE(ival)
}

func EvalAsBoolE(e Expr, data map[string]interface{}) (bool, error) {
	ival, err := e.Eval(data)
	if err != nil {
		return false, err
	}
	return ToBoolE(ival)
}

// From html/template/content.go
// Copyright 2011 The Go Authors. All rights reserved.
// indirect returns the value, after dereferencing as many times
// as necessary to reach the base type (or nil).
func indirect(a interface{}) interface{} {
	if a == nil {
		return nil
	}
	if t := reflect.TypeOf(a); t.Kind() != reflect.Ptr {
		// Avoid creating a reflect.Value if it's not a pointer.
		return a
	}
	v := reflect.ValueOf(a)
	for v.Kind() == reflect.Ptr && !v.IsNil() {
		v = v.Elem()
	}
	return v.Interface()
}

// ToBoolE casts an interface to a bool type.
func ToBoolE(i interface{}) (bool, error) {
	i = indirect(i)

	switch b := i.(type) {
	case bool:
		return b, nil
	case nil:
		return false, nil
	case string:
		return strconv.ParseBool(b)
	default:
		return false, fmt.Errorf("unable to cast %#v of type %T to bool", i, i)
	}
}
