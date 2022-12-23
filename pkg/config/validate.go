package config

import (
	"fmt"
	"reflect"
	"strings"

	"github.com/fatih/structs"
)

var ok_empty = []reflect.Kind{reflect.Map, reflect.Slice, reflect.Array, reflect.Ptr, reflect.Bool}

func isOkEmpty(k reflect.Kind) bool {
	for _, ok := range ok_empty {
		if k == ok {
			return true
		}
	}
	return false
}

func FindUninitialized(obj any, defaults ...any) []string {
	uninitialized := []string{}
	for _, field := range structs.Fields(obj) {
		if field.IsExported() && field.IsZero() && !isOkEmpty(field.Kind()) && !IsDefaultZero(field.Name(), defaults...) {

			tag := field.Tag("yaml")
			if tag == "" {
				tag = field.Name()
			}
			uninitialized = append(uninitialized, fmt.Sprintf("%s (%s)", tag, field.Kind().String()))
		}
	}
	return uninitialized
}

func IsDefaultZero(field string, defaults ...any) bool {
	for _, obj := range defaults {
		if structs.IsStruct(obj) {
			field := structs.New(obj).Field(field)
			if field.IsZero() {
				return true
			}
		}
	}
	return false
}

func AssertInitialized(obj any, defaults ...any) error {
	fields := FindUninitialized(obj, defaults...)
	if len(fields) > 0 {
		plural := []string{"", "is"}
		if len(fields) > 1 {
			plural = []string{"s", "are"}
		}
		return fmt.Errorf("yaml deserialization failed, field%s %s %s required", plural[0], strings.Join(fields, ", "), plural[1])
	}
	return nil
}
