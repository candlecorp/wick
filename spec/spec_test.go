package spec_test

import (
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/spec"
)

func TestNamespace(t *testing.T) {
	expectedBytes, err := os.ReadFile("testdata/expected.json")
	if err != nil {
		t.FailNow()
	}

	arg1 := spec.NewArgument("arg1", "val1")
	assert.Equal(t, "val1", arg1.ValueString())

	anno := spec.NewAnnotation("test").
		AddArguments(
			arg1,
			spec.NewArgument("arg2", "val2"))

	annoMap := anno.ToMap()
	assert.Equal(t, map[string]interface{}{
		"arg1": "val1",
		"arg2": "val2",
	}, annoMap)

	var argStruct struct {
		Arg1 string `mapstructure:"arg1"`
		Arg2 string `mapstructure:"arg2"`
	}
	if assert.NoError(t, anno.ToStruct(&argStruct)) {
		assert.Equal(t, "val1", argStruct.Arg1)
		assert.Equal(t, "val2", argStruct.Arg2)
	}

	arg1Argument, ok := anno.Argument("arg1")
	if assert.True(t, ok) {
		assert.Same(t, arg1, arg1Argument)
	}

	dogBreed := spec.NewEnum("DogBreed", "Enumeration for the type of animal").
		AddAnnotations(anno).
		AddAnnotation(anno). // Should not be added twice
		AddValues(
			spec.NewEnumValue("corgi", "Pembroke Welsh Corgi", "Pembroke Welsh Corgi", 1).
				AddAnnotations(anno).
				AddAnnotation(anno), // Should not be added twice
			spec.NewEnumValue("golden_retriever", "Golden Retriever", "Golden Retriever", 2).
				AddAnnotations(anno))

	name := spec.NewField("name", "The pet's name", &spec.TypeRef{
		Kind: spec.KindString,
	}, nil).
		AddAnnotations(anno).
		AddAnnotation(anno)

	ns := spec.NewNamespace("greetings.v1")

	dog := spec.NewType(ns, "Dog", "A dog").
		AddAnnotations(anno).
		AddAnnotation(anno). // Should not be added twice
		AddFields(
			name,
			spec.NewField("breed", "The dog's breed", &spec.TypeRef{
				Kind: spec.KindEnum,
				Enum: dogBreed,
			}, nil).
				AddAnnotations(anno),
			spec.NewField("parentDogIds", "The dog's parents", &spec.TypeRef{
				Kind: spec.KindList,
				ItemType: &spec.TypeRef{
					Kind: spec.KindString,
				},
			}, nil),
			spec.NewField("birthDate", "The dog's birth date", &spec.TypeRef{
				Kind: spec.KindOptional,
				OptionalType: &spec.TypeRef{
					Kind: spec.KindDateTime,
				},
			}, nil),
			spec.NewField("diet", "The dog's food intake", &spec.TypeRef{
				Kind: spec.KindMap,
				KeyType: &spec.TypeRef{
					Kind: spec.KindString,
				},
				ValueType: &spec.TypeRef{
					Kind: spec.KindString,
				},
			}, nil),
		)

	nameField, ok := dog.Field("name")
	if assert.True(t, ok) {
		assert.Same(t, name, nameField)
	}

	cat := spec.NewType(ns, "Cat", "A cat").
		AddAnnotations(anno).
		AddAnnotation(anno). // Should not be added twice
		AddFields(
			name,
			spec.NewField("liveRemaining", "How many lives are remaiing", &spec.TypeRef{
				Kind: spec.KindU8,
			}, nil),
		)

	animal := spec.NewUnion("Animal", "A union of animal types").
		AddTypes(&spec.TypeRef{
			Kind: spec.KindType,
			Type: dog,
		}).
		AddType(&spec.TypeRef{
			Kind: spec.KindType,
			Type: cat,
		}).
		AddAnnotations(anno).
		AddAnnotation(anno)

	response := spec.NewType(ns, "Response", "Greeting response").
		AddFields(
			spec.NewField("message", "The greeting message", &spec.TypeRef{
				Kind: spec.KindString,
			}, nil),
		).AddAnnotations(anno).
		AddAnnotations(anno)

	sayHello := spec.NewOperation("sayHello", "Say hello", true,
		spec.NewType(ns, "sayHelloArgs", "arguments for sayHello").
			AddFields(
				spec.NewField("name", "Name of the person to greet", &spec.TypeRef{
					Kind: spec.KindString,
				}, "World"),
			),
		&spec.TypeRef{
			Kind: spec.KindType,
			Type: response,
		}).
		AddAnnotations(anno).
		AddAnnotation(anno)

	getAnimal := spec.NewOperation("getAnimal", "Retrieve an animal", false,
		spec.NewType(ns, "getAnimalArgs", "arguments for sayHello").
			AddFields(
				spec.NewField("animalId", "ID of the animal", &spec.TypeRef{
					Kind: spec.KindString,
				}, nil),
			),
		&spec.TypeRef{
			Kind:  spec.KindUnion,
			Union: animal,
		}).
		AddAnnotations(anno).
		AddAnnotation(anno)

	service := spec.NewService("Hello", "Greetings").
		AddAnnotations(
			spec.NewAnnotation("service")).
		AddAnnotation(anno).
		AddOperations(sayHello, getAnimal)

	testAnno, ok := service.Annotation("test")
	if assert.True(t, ok) {
		assert.Same(t, anno, testAnno)
	}

	oper, ok := service.Operation("sayHello")
	if assert.True(t, ok) {
		assert.Same(t, sayHello, oper)
	}

	ns.
		AddAnnotations(
			spec.NewAnnotation("anno").AddArguments(
				spec.NewArgument("arg1", "val1"),
				spec.NewArgument("arg2", "val2")),
		).
		AddAnnotation(anno).
		AddServices(service).
		AddTypes(response, dog, cat).
		AddUnions(animal).
		AddEnums(dogBreed)

	dogType, ok := ns.Type("Dog")
	if assert.True(t, ok) {
		assert.Same(t, dog, dogType)
	}

	animalUnion, ok := ns.Union("Animal")
	if assert.True(t, ok) {
		assert.Same(t, animal, animalUnion)
	}

	breedEnum, ok := ns.Enum("DogBreed")
	if assert.True(t, ok) {
		assert.Same(t, dogBreed, breedEnum)
	}

	namespaces := spec.Namespaces{}.AddNamespaces(ns)

	oper, ok = namespaces.Operation("greetings.v1", "Hello", "sayHello")
	if assert.True(t, ok) {
		assert.Same(t, sayHello, oper)
	}

	serv, ok := ns.Service("Hello")
	if assert.True(t, ok) {
		assert.Same(t, service, serv)
	}

	actualBytes, err := json.MarshalIndent(ns, "", "  ")
	require.NoError(t, err)
	fmt.Println(string(actualBytes))

	var expected, actual interface{}
	require.NoError(t, json.Unmarshal(expectedBytes, &expected))
	require.NoError(t, json.Unmarshal(actualBytes, &actual))

	assert.Equal(t, expected, actual)
}

func TestCoalesce(t *testing.T) {
	ns := spec.NewNamespace("test")
	nested := spec.NewType(ns, "Nested", "").
		AddFields(
			spec.NewField("stringField", "", &spec.TypeRef{
				Kind: spec.KindString,
			}, nil),
		)

	parent := spec.NewType(ns, "Parent", "").
		AddFields(
			spec.NewField("boolField", "", &spec.TypeRef{
				Kind: spec.KindBool,
			}, nil),
			spec.NewField("bytesField", "", &spec.TypeRef{
				Kind: spec.KindBytes,
			}, nil),
			spec.NewField("dateTimeField", "", &spec.TypeRef{
				Kind: spec.KindDateTime,
			}, nil),
			spec.NewField("f32Field", "", &spec.TypeRef{
				Kind: spec.KindF32,
			}, nil),
			spec.NewField("f64Field", "", &spec.TypeRef{
				Kind: spec.KindF64,
			}, nil),
			spec.NewField("i8Field", "", &spec.TypeRef{
				Kind: spec.KindI8,
			}, nil),
			spec.NewField("i16Field", "", &spec.TypeRef{
				Kind: spec.KindI16,
			}, nil),
			spec.NewField("i32Field", "", &spec.TypeRef{
				Kind: spec.KindI32,
			}, nil),
			spec.NewField("i64Field", "", &spec.TypeRef{
				Kind: spec.KindI64,
			}, nil),
			spec.NewField("u8Field", "", &spec.TypeRef{
				Kind: spec.KindU8,
			}, nil),
			spec.NewField("u16Field", "", &spec.TypeRef{
				Kind: spec.KindU16,
			}, nil),
			spec.NewField("u32Field", "", &spec.TypeRef{
				Kind: spec.KindU32,
			}, nil),
			spec.NewField("u64Field", "", &spec.TypeRef{
				Kind: spec.KindU64,
			}, nil),
			spec.NewField("nestedField", "", &spec.TypeRef{
				Kind: spec.KindOptional,
				OptionalType: &spec.TypeRef{
					Kind: spec.KindType,
					Type: nested,
				},
			}, nil),
		)

	expected := map[string]interface{}{
		"boolField":     true,
		"bytesField":    []byte(`Hello, test`),
		"dateTimeField": "2021-11-08T09:36:00-05:00",
		"f32Field":      float32(1.1),
		"f64Field":      float64(2.2),
		"i8Field":       int8(127),
		"i16Field":      int16(32767),
		"i32Field":      int32(32768),
		"i64Field":      int64(2147483648),
		"u8Field":       uint8(255),
		"u16Field":      uint16(65535),
		"u32Field":      uint32(4294967295),
		"u64Field":      uint64(9223372036854775807),
		"nestedField": map[string]interface{}{
			"stringField": "1234",
		},
	}
	parentMap := map[string]interface{}{
		"boolField":     "true",
		"bytesField":    "SGVsbG8sIHRlc3Q=",
		"dateTimeField": "2021-11-08T09:36:00-05:00",
		"f32Field":      "1.1",
		"f64Field":      "2.2",
		"i8Field":       "127",
		"i16Field":      "32767",
		"i32Field":      "32768",
		"i64Field":      "2147483648",
		"u8Field":       "255",
		"u16Field":      "65535",
		"u32Field":      "4294967295",
		"u64Field":      "9223372036854775807",
		"nestedField": map[string]interface{}{
			"stringField": 1234,
		},
	}
	err := parent.Coalesce(parentMap, true)
	require.NoError(t, err)
	assert.Equal(t, expected, parentMap)
}

func TestKind(t *testing.T) {
	tests := []struct {
		kind      spec.Kind
		name      string
		primitive bool
	}{
		{spec.KindOptional, "optional", false},
		{spec.KindList, "list", false},
		{spec.KindMap, "map", false},
		{spec.KindString, "string", true},
		{spec.KindU64, "u64", true},
		{spec.KindU32, "u32", true},
		{spec.KindU16, "u16", true},
		{spec.KindU8, "u8", true},
		{spec.KindI64, "i64", true},
		{spec.KindI32, "i32", true},
		{spec.KindI16, "i16", true},
		{spec.KindI8, "i8", true},
		{spec.KindF64, "f64", true},
		{spec.KindF32, "f32", true},
		{spec.KindBool, "bool", true},
		{spec.KindBytes, "bytes", true},
		{spec.KindRaw, "raw", false},
		{spec.KindDateTime, "datetime", true},
		{spec.KindType, "type", false},
		{spec.KindEnum, "enum", false},
		{spec.KindUnion, "union", false},
		{spec.Kind(9999), "unknown", false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.primitive, tt.kind.IsPrimitive())
			assert.Equal(t, tt.name, tt.kind.String())
			b, err := tt.kind.MarshalJSON()
			if assert.NoError(t, err) {
				assert.Equal(t, []byte(`"`+tt.name+`"`), b)
			}
		})
	}
}
