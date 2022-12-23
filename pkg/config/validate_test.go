package config

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

type Test struct {
	Str       string
	Ptr       *string
	List      []string
	Int       int
	Fields    map[string]interface{}
	Child     Inner
	ChildList []Inner
}

type Inner struct {
	Str string
}

func TestValidate(t *testing.T) {
	obj := Test{}
	fields := FindUninitialized(obj)
	assert.Equal(t, len(fields), 3)

	obj = Test{Str: "Hello testers!", Int: 1, Child: Inner{Str: "substr"}}
	fields = FindUninitialized(obj)
	assert.Equal(t, len(fields), 0)

	// Validating children in lists doesn't work yet...
	// obj = Test{Str: "Hello testers!", Child: Inner{Str: "substr"}, ChildList: []Inner{{}}}
	// fields = FindUninitialized(obj)
	// assert.Equal(t, len(fields), 1)
}

func TestValidateWithDefaults(t *testing.T) {
	obj := Test{}
	defaults := Test{Str: "something", Int: 4, Child: Inner{Str: "something"}}

	fields := FindUninitialized(obj, defaults)
	assert.Equal(t, 3, len(fields))

	defaults = Test{Str: "", Int: 0, Child: Inner{Str: ""}}

	fields = FindUninitialized(obj, defaults)
	fmt.Println("Fields uninitialized")
	fmt.Println(fields)
	assert.Equal(t, 0, len(fields))
}

type DefaultsOk struct {
	StrOkZero    string
	StrSomething string
	IntOkZero    int
	IntSomething int
}

func TestDefaultZero(t *testing.T) {
	defaulted := DefaultsOk{
		StrOkZero:    "",
		StrSomething: "Something",
		IntOkZero:    0,
		IntSomething: 42,
	}

	defaultZero := IsDefaultZero("StrOkZero", defaulted)
	assert.True(t, defaultZero)
	defaultZero = IsDefaultZero("StrSomething", defaulted)
	assert.False(t, defaultZero)
	defaultZero = IsDefaultZero("IntOkZero", defaulted)
	assert.True(t, defaultZero)
	defaultZero = IsDefaultZero("IntSomething", defaulted)
	assert.False(t, defaultZero)
}
