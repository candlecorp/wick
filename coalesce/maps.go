package coalesce

import "fmt"

func ToMapSI(v interface{}, recursive bool) (map[string]interface{}, bool) {
	switch t := v.(type) {
	case map[interface{}]interface{}:
		return MapIItoSI(t, recursive), true
	case map[string]interface{}:
		if !recursive {
			return t, true
		}
		for k, v := range t {
			t[k] = ValueIItoSI(v, recursive)
		}
		return t, true
	case map[string]string:
		return MapSStoSI(t), true
	}

	return nil, false
}

func MapIItoSI(m map[interface{}]interface{}, recursive bool) map[string]interface{} {
	ret := make(map[string]interface{}, len(m))
	for k, v := range m {
		if recursive {
			v = ValueIItoSI(v, recursive)
		}
		ret[interfaceToString(k)] = v
	}
	return ret
}

func ValueIItoSI(value interface{}, recursive bool) interface{} {
	switch t := value.(type) {
	case map[interface{}]interface{}:
		value = MapIItoSI(t, recursive)
	case map[string]string:
		value = MapSStoSI(t)
	case []interface{}:
		for i := range t {
			t[i] = ValueIItoSI(t[i], recursive)
		}
	}
	return value
}

func MapSStoSI(m map[string]string) map[string]interface{} {
	ret := make(map[string]interface{}, len(m))
	for k, v := range m {
		ret[interfaceToString(k)] = v
	}
	return ret
}

func interfaceToString(value interface{}) string {
	if s, ok := value.(string); ok {
		return s
	}
	if s, ok := value.(fmt.Stringer); ok {
		return s.String()
	}
	return fmt.Sprintf("%v", value)
}
