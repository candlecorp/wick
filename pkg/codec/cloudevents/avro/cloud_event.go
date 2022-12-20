// Code generated by github.com/actgardner/gogen-avro/v10. DO NOT EDIT.
/*
 * SOURCE:
 *     spec.avsc
 */
package avro

import (
	"encoding/json"
	"fmt"
	"io"

	"github.com/actgardner/gogen-avro/v10/compiler"
	"github.com/actgardner/gogen-avro/v10/vm"
	"github.com/actgardner/gogen-avro/v10/vm/types"
)

var _ = fmt.Printf

// Avro Event Format for CloudEvents
type CloudEvent struct {
	Attribute map[string]*UnionNullBoolIntStringBytes `json:"attribute"`

	Data *UnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleString `json:"data"`
}

const CloudEventAvroCRC64Fingerprint = "\xbb\x95\x12\xfe_\x12Tq"

func NewCloudEvent() CloudEvent {
	r := CloudEvent{}
	r.Attribute = make(map[string]*UnionNullBoolIntStringBytes)

	r.Data = NewUnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleString()

	return r
}

func DeserializeCloudEvent(r io.Reader) (CloudEvent, error) {
	t := NewCloudEvent()
	deser, err := compiler.CompileSchemaBytes([]byte(t.Schema()), []byte(t.Schema()))
	if err != nil {
		return t, err
	}

	err = vm.Eval(r, deser, &t)
	return t, err
}

func DeserializeCloudEventFromSchema(r io.Reader, schema string) (CloudEvent, error) {
	t := NewCloudEvent()

	deser, err := compiler.CompileSchemaBytes([]byte(schema), []byte(t.Schema()))
	if err != nil {
		return t, err
	}

	err = vm.Eval(r, deser, &t)
	return t, err
}

func writeCloudEvent(r CloudEvent, w io.Writer) error {
	var err error
	err = writeMapUnionNullBoolIntStringBytes(r.Attribute, w)
	if err != nil {
		return err
	}
	err = writeUnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleString(r.Data, w)
	if err != nil {
		return err
	}
	return err
}

func (r CloudEvent) Serialize(w io.Writer) error {
	return writeCloudEvent(r, w)
}

func (r CloudEvent) Schema() string {
	return "{\"doc\":\"Avro Event Format for CloudEvents\",\"fields\":[{\"name\":\"attribute\",\"type\":{\"type\":\"map\",\"values\":[\"null\",\"boolean\",\"int\",\"string\",\"bytes\"]}},{\"name\":\"data\",\"type\":[\"bytes\",\"null\",\"boolean\",{\"type\":\"map\",\"values\":[\"null\",\"boolean\",{\"doc\":\"Representation of a JSON Value\",\"fields\":[{\"name\":\"value\",\"type\":{\"type\":\"map\",\"values\":[\"null\",\"boolean\",{\"type\":\"map\",\"values\":\"io.cloudevents.CloudEventData\"},{\"items\":\"io.cloudevents.CloudEventData\",\"type\":\"array\"},\"double\",\"string\"]}}],\"name\":\"CloudEventData\",\"type\":\"record\"},\"double\",\"string\"]},{\"items\":\"io.cloudevents.CloudEventData\",\"type\":\"array\"},\"double\",\"string\"]}],\"name\":\"io.cloudevents.CloudEvent\",\"type\":\"record\",\"version\":\"1.0\"}"
}

func (r CloudEvent) SchemaName() string {
	return "io.cloudevents.CloudEvent"
}

func (_ CloudEvent) SetBoolean(v bool)    { panic("Unsupported operation") }
func (_ CloudEvent) SetInt(v int32)       { panic("Unsupported operation") }
func (_ CloudEvent) SetLong(v int64)      { panic("Unsupported operation") }
func (_ CloudEvent) SetFloat(v float32)   { panic("Unsupported operation") }
func (_ CloudEvent) SetDouble(v float64)  { panic("Unsupported operation") }
func (_ CloudEvent) SetBytes(v []byte)    { panic("Unsupported operation") }
func (_ CloudEvent) SetString(v string)   { panic("Unsupported operation") }
func (_ CloudEvent) SetUnionElem(v int64) { panic("Unsupported operation") }

func (r *CloudEvent) Get(i int) types.Field {
	switch i {
	case 0:
		r.Attribute = make(map[string]*UnionNullBoolIntStringBytes)

		w := MapUnionNullBoolIntStringBytesWrapper{Target: &r.Attribute}

		return &w

	case 1:
		r.Data = NewUnionBytesNullBoolMapUnionNullBoolCloudEventDataDoubleStringArrayCloudEventDataDoubleString()

		return r.Data
	}
	panic("Unknown field index")
}

func (r *CloudEvent) SetDefault(i int) {
	switch i {
	}
	panic("Unknown field index")
}

func (r *CloudEvent) NullField(i int) {
	switch i {
	case 1:
		r.Data = nil
		return
	}
	panic("Not a nullable field index")
}

func (_ CloudEvent) AppendMap(key string) types.Field { panic("Unsupported operation") }
func (_ CloudEvent) AppendArray() types.Field         { panic("Unsupported operation") }
func (_ CloudEvent) HintSize(int)                     { panic("Unsupported operation") }
func (_ CloudEvent) Finalize()                        {}

func (_ CloudEvent) AvroCRC64Fingerprint() []byte {
	return []byte(CloudEventAvroCRC64Fingerprint)
}

func (r CloudEvent) MarshalJSON() ([]byte, error) {
	var err error
	output := make(map[string]json.RawMessage)
	output["attribute"], err = json.Marshal(r.Attribute)
	if err != nil {
		return nil, err
	}
	output["data"], err = json.Marshal(r.Data)
	if err != nil {
		return nil, err
	}
	return json.Marshal(output)
}

func (r *CloudEvent) UnmarshalJSON(data []byte) error {
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		return err
	}

	var val json.RawMessage
	val = func() json.RawMessage {
		if v, ok := fields["attribute"]; ok {
			return v
		}
		return nil
	}()

	if val != nil {
		if err := json.Unmarshal([]byte(val), &r.Attribute); err != nil {
			return err
		}
	} else {
		return fmt.Errorf("no value specified for attribute")
	}
	val = func() json.RawMessage {
		if v, ok := fields["data"]; ok {
			return v
		}
		return nil
	}()

	if val != nil {
		if err := json.Unmarshal([]byte(val), &r.Data); err != nil {
			return err
		}
	} else {
		return fmt.Errorf("no value specified for data")
	}
	return nil
}