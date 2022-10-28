package spec

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"reflect"
	"strings"

	ut "github.com/go-playground/universal-translator"
	"github.com/go-playground/validator/v10"
	"github.com/mitchellh/mapstructure"
	"github.com/spf13/cast"

	"github.com/nanobus/nanobus/coalesce"
	"github.com/nanobus/nanobus/registry"
)

type (
	NamedLoader = registry.NamedLoader[[]*Namespace]
	Loader      = registry.Loader[[]*Namespace]
	Registry    = registry.Registry[[]*Namespace]

	// NamedLoader func() (string, Loader)
	// Loader      func(config interface{}) ([]*Namespace, error)
	// Registry    map[string]Loader
)

type (
	Namespaces map[string]*Namespace

	Namespace struct {
		Name string `json:"name"`
		Annotated
		Services       []*Service          `json:"services"`
		servicesByName map[string]*Service `json:"-"`
		Types          []*Type             `json:"types,omitempty"`
		typesByName    map[string]*Type    `json:"-"`
		Enums          []*Enum             `json:"enums,omitempty"`
		enumsByName    map[string]*Enum    `json:"-"`
		Unions         []*Union            `json:"unions,omitempty"`
		unionsByName   map[string]*Union   `json:"-"`
	}

	Service struct {
		Name        string `json:"name"`
		Description string `json:"description,omitempty"`
		Annotated
		Operations       []*Operation          `json:"operations"`
		operationsByName map[string]*Operation `json:"-"`
	}

	Operation struct {
		Name        string `json:"name"`
		Description string `json:"description,omitempty"`
		Annotated
		Unary      bool     `json:"unary"`
		Parameters *Type    `json:"parameters,omitempty"`
		Returns    *TypeRef `json:"returns,omitempty"`
	}

	Type struct {
		Namespace   *Namespace `json:"-"`
		Name        string     `json:"name"`
		Description string     `json:"description,omitempty"`
		Annotated
		Fields       []*Field          `json:"fields"`
		fieldsByName map[string]*Field `json:"-"`

		Validations []Validation      `json:"-"`
		rules       map[string]string `json:"-"`
	}

	Field struct {
		Name        string `json:"name"`
		Description string `json:"description,omitempty"`
		Annotated
		Type         *TypeRef    `json:"type"`
		DefaultValue interface{} `json:"defaultValue,omitempty"`

		Validations []Validation `json:"-"`
		//Rules       string       `json:"-"`
	}

	Enum struct {
		Name        string `json:"name"`
		Description string `json:"description,omitempty"`
		Annotated
		Values       []*EnumValue `json:"values"`
		valuesByName map[string]*EnumValue
	}

	EnumValue struct {
		Name        string `json:"name"`
		Description string `json:"description,omitempty"`
		Annotated
		StringValue string `json:"stringValue"`
		IndexValue  int    `json:"indexValue"`
	}

	Union struct {
		Name        string `json:"name"`
		Description string `json:"description,omitempty"`
		Annotated
		Types []*TypeRef `json:"types"`
	}

	Annotated struct {
		Annotations       []*Annotation          `json:"annotations,omitempty"`
		annotationsByName map[string]*Annotation `json:"-"`
	}

	Annotation struct {
		Name            string               `json:"name"`
		Arguments       []*Argument          `json:"arguments,omitempty"`
		argumentsByName map[string]*Argument `json:"-"`
	}

	Argument struct {
		Name  string      `json:"name"`
		Value interface{} `json:"value"`
	}

	TypeRef struct {
		Kind         Kind
		Type         *Type
		Enum         *Enum
		Union        *Union
		OptionalType *TypeRef
		StreamType   *TypeRef
		ItemType     *TypeRef
		KeyType      *TypeRef
		ValueType    *TypeRef
	}

	ValidationLoader func(t *TypeRef, f *Field, a *Annotation) (Validation, error)
	Validation       func(v interface{}) ([]ValidationError, error)

	ValidationErrors map[string][]ValidationError
	ValidationError  struct {
		Rule    string      `json:"rule"`
		Message string      `json:"message"`
		Value   interface{} `json:"value,omitempty"`
	}

	Annotator interface {
		Annotation(name string) (*Annotation, bool)
	}
)

func (ns Namespaces) AddNamespaces(namespaces ...*Namespace) Namespaces {
	for _, n := range namespaces {
		ns.AddNamespace(n)
	}
	return ns
}

func (ns Namespaces) AddNamespace(namespace *Namespace) Namespaces {
	ns[namespace.Name] = namespace
	return ns
}

func (ns Namespaces) Operation(namespace, service, operation string) (*Operation, bool) {
	n, ok := ns[namespace]
	if !ok {
		return nil, false
	}
	return n.Operation(service, operation)
}

func NewNamespace(name string) *Namespace {
	return &Namespace{
		Name:           name,
		Annotated:      newAnnotated(),
		Services:       make([]*Service, 0, 10),
		servicesByName: map[string]*Service{},
		Types:          make([]*Type, 0, 10),
		typesByName:    make(map[string]*Type),
		Enums:          make([]*Enum, 0, 10),
		enumsByName:    make(map[string]*Enum),
		Unions:         make([]*Union, 0, 10),
		unionsByName:   make(map[string]*Union),
	}
}

func (ns *Namespace) AddServices(services ...*Service) *Namespace {
	for _, s := range services {
		ns.AddService(s)
	}
	return ns
}

func (ns *Namespace) AddService(service *Service) *Namespace {
	if _, exists := ns.servicesByName[service.Name]; exists {
		return ns
	}
	ns.servicesByName[service.Name] = service
	ns.Services = append(ns.Services, service)
	return ns
}

func (ns *Namespace) Service(name string) (*Service, bool) {
	s, ok := ns.servicesByName[name]
	return s, ok
}

func (ns *Namespace) AddTypes(types ...*Type) *Namespace {
	for _, t := range types {
		ns.AddType(t)
	}
	return ns
}

func (ns *Namespace) AddType(t *Type) *Namespace {
	if _, exists := ns.typesByName[t.Name]; exists {
		return ns
	}
	ns.typesByName[t.Name] = t
	ns.Types = append(ns.Types, t)
	return ns
}

func (ns *Namespace) Type(name string) (*Type, bool) {
	s, ok := ns.typesByName[name]
	return s, ok
}

func (ns *Namespace) AddEnums(enums ...*Enum) *Namespace {
	for _, e := range enums {
		ns.AddEnum(e)
	}
	return ns
}

func (ns *Namespace) AddEnum(e *Enum) *Namespace {
	if _, exists := ns.enumsByName[e.Name]; exists {
		return ns
	}
	ns.enumsByName[e.Name] = e
	ns.Enums = append(ns.Enums, e)
	return ns
}

func (ns *Namespace) Enum(name string) (*Enum, bool) {
	e, ok := ns.enumsByName[name]
	return e, ok
}

func (ns *Namespace) AddUnions(unions ...*Union) *Namespace {
	for _, u := range unions {
		ns.AddUnion(u)
	}
	return ns
}

func (ns *Namespace) AddUnion(u *Union) *Namespace {
	if _, exists := ns.unionsByName[u.Name]; exists {
		return ns
	}
	ns.unionsByName[u.Name] = u
	ns.Unions = append(ns.Unions, u)
	return ns
}

func (ns *Namespace) Union(name string) (*Union, bool) {
	e, ok := ns.unionsByName[name]
	return e, ok
}

func (ns *Namespace) AddAnnotations(annotations ...*Annotation) *Namespace {
	ns.Annotated.AddAnnotations(annotations...)
	return ns
}

func (ns *Namespace) AddAnnotation(a *Annotation) *Namespace {
	ns.Annotated.AddAnnotation(a)
	return ns
}

func (ns *Namespace) Operation(service, operation string) (*Operation, bool) {
	s, ok := ns.Service(service)
	if !ok {
		return nil, false
	}
	o, ok := s.Operation(operation)
	return o, ok
}

func NewService(name string, description string) *Service {
	return &Service{
		Name:             name,
		Description:      description,
		Annotated:        newAnnotated(),
		Operations:       make([]*Operation, 0, 10),
		operationsByName: make(map[string]*Operation),
	}
}

func (s *Service) AddOperation(oper *Operation) *Service {
	if _, exists := s.operationsByName[oper.Name]; exists {
		return s
	}
	s.operationsByName[oper.Name] = oper
	s.Operations = append(s.Operations, oper)
	return s
}

func (s *Service) AddOperations(opers ...*Operation) *Service {
	for _, oper := range opers {
		s.AddOperation(oper)
	}
	return s
}

func (s *Service) Operation(name string) (*Operation, bool) {
	oper, ok := s.operationsByName[name]
	return oper, ok
}

func (s *Service) AddAnnotations(annotations ...*Annotation) *Service {
	s.Annotated.AddAnnotations(annotations...)
	return s
}

func (s *Service) AddAnnotation(a *Annotation) *Service {
	s.Annotated.AddAnnotation(a)
	return s
}

func NewOperation(name string, description string, unary bool, parameters *Type, returns *TypeRef) *Operation {
	return &Operation{
		Name:        name,
		Description: description,
		Annotated:   newAnnotated(),
		Unary:       unary,
		Parameters:  parameters,
		Returns:     returns,
	}
}

func (o *Operation) AddAnnotations(annotations ...*Annotation) *Operation {
	o.Annotated.AddAnnotations(annotations...)
	return o
}

func (o *Operation) AddAnnotation(a *Annotation) *Operation {
	o.Annotated.AddAnnotation(a)
	return o
}

func NewType(namespace *Namespace, name string, description string) *Type {
	return &Type{
		Namespace:    namespace,
		Name:         name,
		Description:  description,
		Annotated:    newAnnotated(),
		Fields:       make([]*Field, 0, 10),
		fieldsByName: make(map[string]*Field),
		Validations:  []Validation{},
	}
}

func (t *Type) AddFields(fields ...*Field) *Type {
	for _, field := range fields {
		t.AddField(field)
	}
	return t
}

func (t *Type) AddField(field *Field) *Type {
	if _, exists := t.fieldsByName[field.Name]; exists {
		return t
	}
	t.fieldsByName[field.Name] = field
	t.Fields = append(t.Fields, field)
	return t
}

func (t *Type) Field(name string) (*Field, bool) {
	field, ok := t.fieldsByName[name]
	return field, ok
}

func (t *Type) AddAnnotations(annotations ...*Annotation) *Type {
	t.Annotated.AddAnnotations(annotations...)
	return t
}

func (s *Type) AddAnnotation(a *Annotation) *Type {
	s.Annotated.AddAnnotation(a)
	return s
}

func NewField(name string, description string, t *TypeRef, defaultValue interface{}) *Field {
	return &Field{
		Name:         name,
		Description:  description,
		Type:         t,
		DefaultValue: defaultValue,
		Annotated:    newAnnotated(),
	}
}

func (f *Field) AddAnnotations(annotations ...*Annotation) *Field {
	f.Annotated.AddAnnotations(annotations...)
	return f
}

func (f *Field) AddAnnotation(a *Annotation) *Field {
	f.Annotated.AddAnnotation(a)
	return f
}

func (t *Type) InitValidations() (*Type, error) {
	t.rules = make(map[string]string, len(t.Fields))
	for _, f := range t.Fields {
		var rules []string
		if f.Type.Kind != KindOptional {
			rules = append(rules, "required")
		}
		for _, a := range f.Annotations {
			if v, ok := validationRules[a.Name]; ok {
				tag, err := v(a)
				if err != nil {
					return t, err
				}
				rules = append(rules, tag)
			}
		}
		if len(rules) > 0 {
			t.rules[f.Name] = strings.Join(rules, ",")
		}
	}

	return t, nil
}

func (f *Field) InitValidations() error {
	for _, a := range f.Annotations {
		if v, ok := validators[a.Name]; ok {
			validation, err := v(nil, f, a)
			if err != nil {
				return err
			}
			f.Validations = append(f.Validations, validation)
		}
	}

	return nil
}

func NewEnum(name string, description string) *Enum {
	return &Enum{
		Name:         name,
		Description:  description,
		Annotated:    newAnnotated(),
		Values:       make([]*EnumValue, 0, 10),
		valuesByName: make(map[string]*EnumValue),
	}
}

func (e *Enum) AddValues(values ...*EnumValue) *Enum {
	for _, field := range values {
		e.AddValue(field)
	}
	return e
}

func (e *Enum) AddValue(value *EnumValue) *Enum {
	if _, exists := e.valuesByName[value.Name]; exists {
		return e
	}
	e.valuesByName[value.Name] = value
	e.Values = append(e.Values, value)
	return e
}

func (e *Enum) AddAnnotations(annotations ...*Annotation) *Enum {
	e.Annotated.AddAnnotations(annotations...)
	return e
}

func (e *Enum) AddAnnotation(a *Annotation) *Enum {
	e.Annotated.AddAnnotation(a)
	return e
}

func NewEnumValue(name string, description string, stringValue string, indexValue int) *EnumValue {
	return &EnumValue{
		Name:        name,
		Description: description,
		Annotated:   newAnnotated(),
		StringValue: stringValue,
		IndexValue:  indexValue,
	}
}

func (e *EnumValue) AddAnnotations(annotations ...*Annotation) *EnumValue {
	e.Annotated.AddAnnotations(annotations...)
	return e
}

func (e *EnumValue) AddAnnotation(a *Annotation) *EnumValue {
	e.Annotated.AddAnnotation(a)
	return e
}

func NewUnion(name string, description string) *Union {
	return &Union{
		Name:        name,
		Description: description,
		Annotated:   newAnnotated(),
		Types:       make([]*TypeRef, 0, 10),
	}
}

func (u *Union) AddTypes(types ...*TypeRef) *Union {
	u.Types = append(u.Types, types...)
	return u
}

func (u *Union) AddType(t *TypeRef) *Union {
	u.Types = append(u.Types, t)
	return u
}

func (u *Union) AddAnnotations(annotations ...*Annotation) *Union {
	u.Annotated.AddAnnotations(annotations...)
	return u
}

func (u *Union) AddAnnotation(a *Annotation) *Union {
	u.Annotated.AddAnnotation(a)
	return u
}

func (t *TypeRef) IsPrimitive() bool {
	if t.Kind == KindOptional {
		return t.OptionalType.IsPrimitive()
	}
	return t.Kind.IsPrimitive()
}

func (t *TypeRef) Coalesce(value interface{}, validate bool) (interface{}, bool, error) {
	var err error
	var changed bool
	if value == nil {
		if validate && t.Kind != KindOptional {
			return nil, false, fmt.Errorf("value is required")
		}
		return nil, false, nil
	}
	switch t.Kind {
	case KindOptional:
		if value == nil {
			return nil, false, nil
		}
		return t.OptionalType.Coalesce(value, validate)
	case KindString:
		if _, ok := value.(string); !ok {
			value, err = cast.ToStringE(value)
			changed = true
		}
	case KindDateTime:
		if _, ok := value.(string); !ok {
			err = fmt.Errorf("value must be an string, got %s", reflect.TypeOf(value))
		}
	case KindU64:
		if _, ok := value.(uint64); !ok {
			value, err = cast.ToUint64E(value)
			changed = true
		}
	case KindU32:
		if _, ok := value.(uint32); !ok {
			value, err = cast.ToUint32E(value)
			changed = true
		}
	case KindU16:
		if _, ok := value.(uint16); !ok {
			value, err = cast.ToUint16E(value)
			changed = true
		}
	case KindU8:
		if _, ok := value.(uint8); !ok {
			value, err = cast.ToUint8E(value)
			changed = true
		}
	case KindI64:
		if _, ok := value.(int64); !ok {
			value, err = cast.ToInt64E(value)
			changed = true
		}
	case KindI32:
		if _, ok := value.(int32); !ok {
			value, err = cast.ToInt32E(value)
			changed = true
		}
	case KindI16:
		if _, ok := value.(int16); !ok {
			value, err = cast.ToInt16E(value)
			changed = true
		}
	case KindI8:
		if _, ok := value.(int8); !ok {
			value, err = cast.ToInt8E(value)
			changed = true
		}
	case KindF64:
		if _, ok := value.(float64); !ok {
			value, err = cast.ToFloat64E(value)
			changed = true
		}
	case KindF32:
		if _, ok := value.(float32); !ok {
			value, err = cast.ToFloat32E(value)
			changed = true
		}
	case KindBool:
		if _, ok := value.(bool); !ok {
			value, err = cast.ToBoolE(value)
			changed = true
		}
	case KindBytes:
		if _, ok := value.([]byte); !ok {
			if stringValue, ok := value.(string); ok {
				value, err = base64.StdEncoding.DecodeString(stringValue)
			} else {
				err = fmt.Errorf("value must be a boolean, got %s", reflect.TypeOf(value))
			}
			changed = true
		}
	case KindType:
		valueMap, ok := coalesce.ToMapSI(value, false)
		if !ok {
			err = fmt.Errorf("value must be a map, got %s", reflect.TypeOf(value))
		}
		if err == nil {
			err = t.Type.Coalesce(valueMap, validate)
			changed = true
		}
		value = valueMap
	case KindList:
		switch vv := value.(type) {
		case []interface{}:
			for i, item := range vv {
				var itemChanged bool
				item, itemChanged, err = t.ItemType.Coalesce(item, validate)
				if err != nil {
					return nil, false, err
				}
				if itemChanged {
					changed = true
					vv[i] = item
				}
			}
		case []map[string]interface{}:
			for i, item := range vv {
				var itemChanged bool
				var val interface{}
				val, itemChanged, err = t.ItemType.Coalesce(item, validate)
				if err != nil {
					return nil, false, err
				}
				if itemChanged {
					changed = true
					vv[i] = val.(map[string]interface{})
				}
			}
		// TODO
		// case KindEnum:
		// case KindUnion:
		default:
			err = fmt.Errorf("value must be a slice, got %s", reflect.TypeOf(value))
		}
	}

	return value, changed, err
}

func (t *TypeRef) MarshalJSON() ([]byte, error) {
	rep := t.jsonValue()
	return json.Marshal(rep)
}

func (t *TypeRef) jsonValue() interface{} {
	switch t.Kind {
	case KindEnum:
		return map[string]string{
			"$enum": t.Enum.Name,
		}
	case KindList:
		return map[string]interface{}{
			"$list": t.ItemType.jsonValue(),
		}
	case KindMap:
		return map[string]interface{}{
			"$map": map[string]interface{}{
				"keyType":   t.KeyType.jsonValue(),
				"valueType": t.ValueType.jsonValue(),
			},
		}
	case KindOptional:
		return map[string]interface{}{
			"$optional": t.OptionalType.jsonValue(),
		}
	case KindStream:
		return map[string]interface{}{
			"$stream": t.StreamType.jsonValue(),
		}
	case KindType:
		return map[string]string{
			"$type": t.Type.Name,
		}
	case KindUnion:
		return map[string]string{
			"$union": t.Union.Name,
		}
	}
	return t.Kind.String()
}

func (t *Type) Coalesce(v map[string]interface{}, validate bool) error {
	if v == nil {
		return nil
	}
	for fieldName, value := range v {
		f, ok := t.fieldsByName[fieldName]
		if !ok {
			// Exclude extraneous values.
			delete(v, fieldName)
			continue
		}

		if err := t.doField(f.Type, f, fieldName, v, value, validate); err != nil {
			return err
		}
	}

	if validate {
		// var _verrors [25]ValidationError
		// verrors := _verrors[:0]
		myErrors := make(ValidationErrors, len(t.rules))

		// for fieldName, f := range t.fieldsByName {
		// 	_, ok := v[fieldName]
		// 	if f.Type.Kind != KindOptional && !ok {
		// 		return fmt.Errorf("missing required field %s in type %s", fieldName, t.Name)
		// 	}
		// 	// for _, val := range f.Validations {
		// 	// 	valErrors, err := val(value)
		// 	// 	if err != nil {
		// 	// 		return err
		// 	// 	}
		// 	// 	if len(valErrors) > 0 {
		// 	// 		verrors = append(verrors, valErrors...)
		// 	// 	}
		// 	// }
		// }

		valErrors := validateMapCtx(globalValidate, context.Background(), v, t.rules)
		for name, errs := range valErrors {
			if verrs, ok := errs.(validator.ValidationErrors); ok {
				errors := make([]ValidationError, len(verrs))
				for i, err := range verrs {
					sfe := specFieldError{err, t, name}
					msg := sfe.Translate(translator)
					errors[i] = ValidationError{
						Rule:    err.Tag(),
						Message: msg,
						Value:   err.Value(),
					}
				}
				myErrors[name] = errors
			}
		}

		// if len(myErrors) > 0 {
		// 	msg := "parameter input is invalid"
		// 	if unicode.IsUpper([]rune(t.Name)[0]) {
		// 		msg = fmt.Sprintf("input for %s is invalid", t.Name)
		// 	}
		// 	err := errorz.New(errorz.InvalidArgument, msg)
		// 	err.Details = myErrors
		// 	return err
		// }
	}

	return nil
}

func validateMapCtx(v *validator.Validate, ctx context.Context, data map[string]interface{}, rules map[string]string) map[string]error {
	errs := make(map[string]error, len(rules))
	for field, rule := range rules {
		err := v.VarCtx(ctx, data[field], rule)
		if err != nil {
			errs[field] = err
		}
	}
	return errs
}

type specFieldError struct {
	validator.FieldError
	t     *Type
	field string
}

func (e specFieldError) Namespace() string {
	return e.t.Namespace.Name
}

func (e specFieldError) StructNamespace() string {
	return e.t.Namespace.Name
}

func (e specFieldError) Field() string {
	return e.field
}

func (e specFieldError) StructField() string {
	return e.field
}

func (e specFieldError) Translate(ut ut.Translator) string {
	fn, ok := e.Validate().GetValidatorFunc(ut, e.field)
	if !ok {
		return e.Error()
	}

	return fn(ut, e)
}

func (t *Type) doField(tt *TypeRef, f *Field, fieldName string, v map[string]interface{}, value interface{}, validate bool) (err error) {
	newValue, changed, err := tt.Coalesce(value, validate)
	if err != nil {
		return fmt.Errorf("invalid field %q of type %q: %w", f.Name, t.Name, err)
	}
	if changed {
		v[fieldName] = newValue
	}

	return nil
}

func newAnnotated() Annotated {
	return Annotated{
		Annotations:       make([]*Annotation, 0, 10),
		annotationsByName: make(map[string]*Annotation),
	}
}

func (a *Annotated) AddAnnotations(annotations ...*Annotation) *Annotated {
	for _, an := range annotations {
		a.AddAnnotation(an)
	}
	return a
}

func (a *Annotated) AddAnnotation(an *Annotation) *Annotated {
	if _, exists := a.annotationsByName[an.Name]; exists {
		return a
	}
	a.annotationsByName[an.Name] = an
	a.Annotations = append(a.Annotations, an)
	return a
}

func (a *Annotated) Annotation(name string) (*Annotation, bool) {
	anno, ok := a.annotationsByName[name]
	return anno, ok
}

func NewAnnotation(name string) *Annotation {
	return &Annotation{
		Name:            name,
		Arguments:       make([]*Argument, 0, 10),
		argumentsByName: make(map[string]*Argument),
	}
}

func (a *Annotation) AddArguments(args ...*Argument) *Annotation {
	for _, an := range args {
		a.AddArgument(an)
	}
	return a
}

func (a *Annotation) AddArgument(an *Argument) *Annotation {
	if _, exists := a.argumentsByName[an.Name]; exists {
		return a
	}
	a.argumentsByName[an.Name] = an
	a.Arguments = append(a.Arguments, an)
	return a
}

func (a *Annotation) Argument(name string) (*Argument, bool) {
	arg, ok := a.argumentsByName[name]
	return arg, ok
}

func (a *Annotation) ToMap() map[string]interface{} {
	m := make(map[string]interface{}, len(a.Arguments))
	for name, arg := range a.argumentsByName {
		m[name] = arg.Value
	}
	return m
}

func (a *Annotation) ToStruct(dst interface{}) error {
	return mapstructure.Decode(a.ToMap(), dst)
}

func NewArgument(name string, value interface{}) *Argument {
	return &Argument{
		Name:  name,
		Value: value,
	}
}

func (a *Argument) ValueString() string {
	return cast.ToString(a.Value)
}

type Kind int

const (
	KindUnknown Kind = iota
	KindOptional
	KindStream
	KindList
	KindMap
	KindString
	KindU64
	KindU32
	KindU16
	KindU8
	KindI64
	KindI32
	KindI16
	KindI8
	KindF64
	KindF32
	KindBool
	KindBytes
	KindRaw
	KindDateTime
	KindType
	KindEnum
	KindUnion
)

var kindStringMap = map[Kind]string{
	KindOptional: "optional",
	KindStream:   "stream",
	KindList:     "list",
	KindMap:      "map",
	KindString:   "string",
	KindU64:      "u64",
	KindU32:      "u32",
	KindU16:      "u16",
	KindU8:       "u8",
	KindI64:      "i64",
	KindI32:      "i32",
	KindI16:      "i16",
	KindI8:       "i8",
	KindF64:      "f64",
	KindF32:      "f32",
	KindBool:     "bool",
	KindBytes:    "bytes",
	KindRaw:      "raw",
	KindDateTime: "datetime",
	KindType:     "type",
	KindEnum:     "enum",
	KindUnion:    "union",
}

func (k Kind) String() string {
	if str, ok := kindStringMap[k]; ok {
		return str
	}
	return "unknown"
}

func (k Kind) IsPrimitive() bool {
	switch k {
	case KindString, KindU64, KindU32, KindU16, KindU8, KindI64,
		KindI32, KindI16, KindI8, KindF64, KindF32, KindBool, KindDateTime,
		KindBytes:
		return true
	}
	return false
}

func (k Kind) MarshalJSON() ([]byte, error) {
	s := k.String()
	return json.Marshal(s)
}
