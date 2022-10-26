//go:generate $GOPATH/bin/gogen-avro . spec.avsc
package avro

import (
	"bytes"
	"encoding/json"

	"github.com/actgardner/gogen-avro/v10/compiler"
	"github.com/actgardner/gogen-avro/v10/vm"

	"github.com/nanobus/nanobus/codec"
	"github.com/nanobus/nanobus/resolve"
)

type (
	// Codec encodes and decodes Avro records.
	Codec struct {
		deser *vm.Program
	}
)

// JSON is the NamedLoader for this codec.
func CloudEventsAvro() (string, bool, codec.Loader) {
	return "cloudevents+avro", true, Loader
}

func Loader(with interface{}, resolver resolve.ResolveAs) (codec.Codec, error) {
	t := CloudEvent{}
	deser, err := compiler.CompileSchemaBytes([]byte(t.Schema()), []byte(t.Schema()))
	if err != nil {
		return nil, err
	}

	return NewCodec(deser), nil
}

// NewCodec creates a `Codec`.
func NewCodec(deser *vm.Program) *Codec {
	return &Codec{
		deser: deser,
	}
}

func (c *Codec) ContentType() string {
	return "application/avro"
}

// Decode decodes CloudEvents Avro bytes to a value.
func (c *Codec) Decode(msgValue []byte, args ...interface{}) (interface{}, string, error) {
	t := NewCloudEvent()
	deser, err := compiler.CompileSchemaBytes([]byte(t.Schema()), []byte(t.Schema()))
	if err != nil {
		return nil, "", err
	}

	r := bytes.NewReader(msgValue)

	err = vm.Eval(r, deser, &t)
	if err != nil {
		return nil, "", err
	}

	var eventType string

	event := make(map[string]interface{}, len(t.Attribute)+1)
	for key, value := range t.Attribute {
		var v interface{}
		switch value.UnionType {
		case UnionNullBoolIntStringBytesTypeEnumBool:
			v = value.Bool
		case UnionNullBoolIntStringBytesTypeEnumInt:
			v = value.Int
		case UnionNullBoolIntStringBytesTypeEnumString:
			v = value.String
		case UnionNullBoolIntStringBytesTypeEnumBytes:
			v = value.Bytes
		}
		event[key] = v
	}

	if typeI, ok := event["type"]; ok {
		eventType, _ = typeI.(string)
	}

	switch t.Data.UnionType {
	case UnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleStringTypeEnumBytes:
		event["data"] = t.Data.Bytes
	case UnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleStringTypeEnumBool:
		event["data"] = t.Data.Bool
	case UnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleStringTypeEnumMapUnionNullBoolCloudEventDataDoubleString:
		event["data"] = decodeDataMap(t.Data.MapUnionNullBoolCloudEventDataDoubleString)
	case UnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleStringTypeEnumArrayCloudEventData:
		event["data"] = decodeCloudEventDataArray(t.Data.ArrayCloudEventData)
	case UnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleStringTypeEnumDouble:
		event["data"] = t.Data.Double
	case UnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleStringTypeEnumString:
		event["data"] = t.Data.String
	}

	return event, eventType, err
}

func decodeDataMap(d map[string]*UnionNullBoolCloudEventDataDoubleString) map[string]interface{} {
	m := make(map[string]interface{}, len(d))
	for key, value := range d {
		switch value.UnionType {
		case UnionNullBoolCloudEventDataDoubleStringTypeEnumBool:
			m[key] = value.Bool
		case UnionNullBoolCloudEventDataDoubleStringTypeEnumCloudEventData:
			m[key] = decodeCloudEventData(&value.CloudEventData)
		case UnionNullBoolCloudEventDataDoubleStringTypeEnumDouble:
			m[key] = value.Double
		case UnionNullBoolCloudEventDataDoubleStringTypeEnumString:
			m[key] = value.String
		}
	}
	return m
}

func decodeCloudEventData(d *CloudEventData) map[string]interface{} {
	m := make(map[string]interface{}, len(d.Value))
	for key, value := range d.Value {
		switch value.UnionType {
		case UnionNullBoolMapCloudEventDataArrayCloudEventDataDoubleStringTypeEnumBool:
			m[key] = value.Bool
		case UnionNullBoolMapCloudEventDataArrayCloudEventDataDoubleStringTypeEnumMapCloudEventData:
			m[key] = decodeCloudEventDataMap(value.MapCloudEventData)
		case UnionNullBoolMapCloudEventDataArrayCloudEventDataDoubleStringTypeEnumArrayCloudEventData:
			m[key] = decodeCloudEventDataArray(value.ArrayCloudEventData)
		case UnionNullBoolMapCloudEventDataArrayCloudEventDataDoubleStringTypeEnumDouble:
			m[key] = value.Double
		case UnionNullBoolMapCloudEventDataArrayCloudEventDataDoubleStringTypeEnumString:
			m[key] = value.String
		}
	}
	return m
}

func decodeCloudEventDataArray(d []CloudEventData) []interface{} {
	m := make([]interface{}, len(d))
	for i := range d {
		value := &d[i]
		m[i] = decodeCloudEventData(value)
	}
	return m
}

func decodeCloudEventDataMap(d map[string]CloudEventData) map[string]interface{} {
	m := make(map[string]interface{}, len(d))
	for key, value := range d {
		m[key] = decodeCloudEventData(&value)
	}
	return m
}

// Encode encodes a value into CloudEvents Avro encoded bytes.
func (c *Codec) Encode(value interface{}, args ...interface{}) ([]byte, error) {
	return json.Marshal(value)
}
