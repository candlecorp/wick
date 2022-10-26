package coalesce

import (
	"encoding/json"
	"math"
	"reflect"
)

// JSONUnmarshal wraps json.Unmarshal and also handles
// converting float64 values that can be truncated to integers to int64.
func JSONUnmarshal(data []byte, v interface{}) error {
	if err := json.Unmarshal(data, v); err != nil {
		return err
	}

	rv := reflect.ValueOf(v)
	elem := rv.Elem()
	i := elem.Interface()
	ri := reflect.ValueOf(Integers(i))
	elem.Set(ri)

	return nil
}

// Integers converts float64 values that can be truncated to integers to int64.
// This aids in the conversion of JSON to MessagePack then to data structures.
func Integers(v interface{}) interface{} {
	switch t := v.(type) {
	case float64:
		if t == math.Trunc(t) {
			return int64(t)
		}
	case map[interface{}]interface{}:
		for k, v := range t {
			t[k] = Integers(v)
		}
	case map[string]interface{}:
		for k, v := range t {
			t[k] = Integers(v)
		}
	case []interface{}:
		for i, v := range t {
			t[i] = Integers(v)
		}
	}

	return v
}

func Unsigned(v interface{}) interface{} {
	switch t := v.(type) {
	case uint64:
		return int64(t)
	case uint32:
		return int32(t)
	case uint16:
		return int32(t)
	case uint8:
		return int32(t)
	case int16:
		return int32(t)
	case int8:
		return int32(t)
	case map[interface{}]interface{}:
		for k, v := range t {
			t[k] = Unsigned(v)
		}
	case map[string]interface{}:
		for k, v := range t {
			t[k] = Unsigned(v)
		}
	}

	return v
}
