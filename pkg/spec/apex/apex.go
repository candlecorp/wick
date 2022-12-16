/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package apex

import (
	"context"
	"os"
	"strings"

	"github.com/apexlang/apex-go/ast"
	"github.com/apexlang/apex-go/parser"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/runtime"
	"github.com/nanobus/nanobus/pkg/spec"
)

type Config struct {
	// Filename is the file name of the Apex definition to load.
	// TODO: Load from external location
	Filename runtime.FilePath `mapstructure:"filename" validate:"required"`
}

// Apex is the NamedLoader for the Apex spec.
func Apex() (string, spec.Loader) {
	return "apex", Loader
}

func Loader(ctx context.Context, with interface{}, resolveAs resolve.ResolveAs) ([]*spec.Namespace, error) {
	c := Config{
		Filename: "apex.axdl",
	}

	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	specBytes, err := os.ReadFile(string(c.Filename))
	if err != nil {
		return nil, err
	}

	ns, err := Parse(specBytes)
	if err != nil {
		return nil, err
	}

	return []*spec.Namespace{ns}, nil
}

type nsParser struct {
	n       *spec.Namespace
	aliases map[string]*ast.AliasDefinition
}

func Parse(schema []byte) (*spec.Namespace, error) {
	n := spec.NewNamespace("")
	doc, err := parser.Parse(parser.ParseParams{
		Source: string(schema),
		Options: parser.ParseOptions{
			NoLocation: true,
			NoSource:   true,
			Resolver: func(location string, from string) (string, error) {
				if strings.HasPrefix(location, "@") {
					// what do?
					return "", nil
				} else {
					src, err := os.ReadFile(location)
					return string(src), err
				}
			},
		},
	})
	if err != nil {
		return nil, err
	}

	p := nsParser{
		n:       n,
		aliases: make(map[string]*ast.AliasDefinition),
	}

	for _, def := range doc.Definitions {
		switch d := def.(type) {
		case *ast.NamespaceDefinition:
			n.Name = d.Name.Value
			n.AddAnnotations(p.convertAnnotations(d.Annotations)...)

		case *ast.TypeDefinition:
			// Create a placeholder for the type in memory
			n.AddType(spec.NewType(n, d.Name.Value, stringValue(d.Description)))

		case *ast.EnumDefinition:
			// Create a placeholder for the enum in memory
			n.AddEnum(spec.NewEnum(d.Name.Value, stringValue(d.Description)))

		case *ast.UnionDefinition:
			// Create a placeholder for the enum in memory
			n.AddUnion(spec.NewUnion(d.Name.Value, stringValue(d.Description)))

		case *ast.AliasDefinition:
			p.aliases[d.Name.Value] = d
		}
	}

	for _, def := range doc.Definitions {
		switch d := def.(type) {
		case *ast.TypeDefinition:
			// Populate the type information
			t, err := p.createType(d)
			if err != nil {
				return nil, err
			}
			n.AddType(t)
		case *ast.EnumDefinition:
			// Populate the enum information
			n.AddEnum(p.createEnum(d))
		case *ast.UnionDefinition:
			// Populate the union information
			n.AddUnion(p.createUnion(d))
		}
	}

	for _, def := range doc.Definitions {
		switch d := def.(type) {
		case *ast.InterfaceDefinition:
			s, err := p.convertService(d)
			if err != nil {
				return nil, err
			}
			n.AddService(s)
		}
	}

	return n, nil
}

func (p *nsParser) createType(t *ast.TypeDefinition) (*spec.Type, error) {
	tt, ok := p.n.Type(t.Name.Value)
	if !ok {
		tt = spec.NewType(p.n, t.Name.Value, stringValue(t.Description))
	}
	return tt.
		AddFields(p.convertFields(t.Fields)...).
		AddAnnotations(p.convertAnnotations(t.Annotations)...).
		InitValidations()
}

func (p *nsParser) convertFields(fields []*ast.FieldDefinition) []*spec.Field {
	if fields == nil {
		return nil
	}

	o := make([]*spec.Field, len(fields))
	for i, field := range fields {
		var dv interface{}
		if field.Default != nil {
			dv = field.Default.GetValue()
		}
		o[i] = spec.NewField(
			field.Name.Value,
			stringValue(field.Description),
			p.convertTypeRef(field.Type),
			dv).
			AddAnnotations(p.convertAnnotations(field.Annotations)...)
		o[i].InitValidations()
	}

	return o
}

func (p *nsParser) createEnum(t *ast.EnumDefinition) *spec.Enum {
	e, ok := p.n.Enum(t.Name.Value)
	if !ok {
		e = spec.NewEnum(t.Name.Value, stringValue(t.Description))
	}
	return e.
		AddValues(p.convertEnumValues(t.Values)...).
		AddAnnotations(p.convertAnnotations(t.Annotations)...)
}

func (p *nsParser) convertEnumValues(fields []*ast.EnumValueDefinition) []*spec.EnumValue {
	if fields == nil {
		return nil
	}

	o := make([]*spec.EnumValue, len(fields))
	for i, field := range fields {
		o[i] = spec.NewEnumValue(
			field.Name.Value,
			stringValue(field.Description),
			stringValue(field.Display),
			field.Index.Value).
			AddAnnotations(p.convertAnnotations(field.Annotations)...)
	}

	return o
}

func (p *nsParser) createUnion(t *ast.UnionDefinition) *spec.Union {
	e, ok := p.n.Union(t.Name.Value)
	if !ok {
		e = spec.NewUnion(t.Name.Value, stringValue(t.Description))
	}
	for _, t := range t.Types {
		e.AddType(p.convertTypeRef(t))
	}
	return e.AddAnnotations(p.convertAnnotations(t.Annotations)...)
}

func (p *nsParser) convertService(role *ast.InterfaceDefinition) (*spec.Service, error) {
	opers, err := p.convertOperations(role.Operations)
	if err != nil {
		return nil, err
	}
	return spec.NewService(
		role.Name.Value,
		stringValue(role.Description)).
		AddOperations(opers...).
		AddAnnotations(p.convertAnnotations(role.Annotations)...), nil
}

func (p *nsParser) convertOperations(operations []*ast.OperationDefinition) ([]*spec.Operation, error) {
	if operations == nil {
		return nil, nil
	}

	o := make([]*spec.Operation, len(operations))
	for i, operation := range operations {
		var params *spec.Type
		if operation.Unary {
			param := operation.Parameters[0]
			if named, ok := param.Type.(*ast.Named); ok {
				if pt, ok := p.n.Type(named.Name.Value); ok {
					params = pt
				}
			}
		} else {
			var err error
			params, err = p.convertParameterType(operation.Name.Value+"Params", operation.Parameters)
			if err != nil {
				return nil, err
			}
		}
		o[i] = spec.NewOperation(
			operation.Name.Value,
			stringValue(operation.Description),
			operation.Unary,
			params,
			p.convertTypeRef(operation.Type)).
			AddAnnotations(p.convertAnnotations(operation.Annotations)...)
	}

	return o, nil
}

func (p *nsParser) convertParameterType(name string, params []*ast.ParameterDefinition) (*spec.Type, error) {
	fields := p.convertParameters(params)
	return spec.NewType(p.n, name, "").
		AddFields(fields...).
		InitValidations()
}

func (p *nsParser) convertParameters(parameters []*ast.ParameterDefinition) []*spec.Field {
	if parameters == nil {
		return nil
	}

	o := make([]*spec.Field, len(parameters))
	for i, parameter := range parameters {
		var dv interface{}
		if parameter.Default != nil {
			dv = parameter.Default.GetValue()
		}
		o[i] = spec.NewField(
			parameter.Name.Value,
			stringValue(parameter.Description),
			p.convertTypeRef(parameter.Type),
			dv).
			AddAnnotations(p.convertAnnotations(parameter.Annotations)...)
		o[i].InitValidations()
	}

	return o
}

func (p *nsParser) convertAnnotations(annotations []*ast.Annotation) []*spec.Annotation {
	a := make([]*spec.Annotation, len(annotations))
	for i, annotation := range annotations {
		a[i] = spec.NewAnnotation(annotation.Name.Value).
			AddArguments(p.convertArguments(annotation.Arguments)...)
	}

	return a
}

func (p *nsParser) convertArguments(arguments []*ast.Argument) []*spec.Argument {
	if arguments == nil {
		return nil
	}

	a := make([]*spec.Argument, len(arguments))
	for i, argument := range arguments {
		a[i] = &spec.Argument{
			Name:  argument.Name.Value,
			Value: argument.Value.GetValue(),
		}
	}

	return a
}

var (
	typeRefString   = spec.TypeRef{Kind: spec.KindString}
	typeRefU64      = spec.TypeRef{Kind: spec.KindU64}
	typeRefU32      = spec.TypeRef{Kind: spec.KindU32}
	typeRefU16      = spec.TypeRef{Kind: spec.KindU16}
	typeRefU8       = spec.TypeRef{Kind: spec.KindU8}
	typeRefI64      = spec.TypeRef{Kind: spec.KindI64}
	typeRefI32      = spec.TypeRef{Kind: spec.KindI32}
	typeRefI16      = spec.TypeRef{Kind: spec.KindI16}
	typeRefI8       = spec.TypeRef{Kind: spec.KindI8}
	typeRefF64      = spec.TypeRef{Kind: spec.KindF64}
	typeRefF32      = spec.TypeRef{Kind: spec.KindF32}
	typeRefBool     = spec.TypeRef{Kind: spec.KindBool}
	typeRefBytes    = spec.TypeRef{Kind: spec.KindBytes}
	typeRefRaw      = spec.TypeRef{Kind: spec.KindRaw}
	typeRefDateTime = spec.TypeRef{Kind: spec.KindDateTime}

	typeRefMap = map[string]*spec.TypeRef{
		"string":   &typeRefString,
		"u64":      &typeRefU64,
		"u32":      &typeRefU32,
		"u16":      &typeRefU16,
		"u8":       &typeRefU8,
		"i64":      &typeRefI64,
		"i32":      &typeRefI32,
		"i16":      &typeRefI16,
		"i8":       &typeRefI8,
		"f64":      &typeRefF64,
		"f32":      &typeRefF32,
		"bool":     &typeRefBool,
		"bytes":    &typeRefBytes,
		"raw":      &typeRefRaw,
		"datetime": &typeRefDateTime,
	}
)

func (p *nsParser) convertTypeRef(t ast.Type) *spec.TypeRef {
	if t == nil {
		return nil
	}

	switch tt := t.(type) {
	case *ast.Named:
		if tt.Name.Value == "void" {
			return nil
		}
		if prim, ok := typeRefMap[tt.Name.Value]; ok {
			return prim
		}
		if t, ok := p.n.Type(tt.Name.Value); ok {
			return &spec.TypeRef{
				Kind: spec.KindType,
				Type: t,
			}
		}
		if e, ok := p.n.Enum(tt.Name.Value); ok {
			return &spec.TypeRef{
				Kind: spec.KindEnum,
				Enum: e,
			}
		}
		if u, ok := p.n.Union(tt.Name.Value); ok {
			return &spec.TypeRef{
				Kind:  spec.KindUnion,
				Union: u,
			}
		}
		if a, ok := p.aliases[tt.Name.Value]; ok {
			return p.convertTypeRef(a.Type)
		}
	case *ast.ListType:
		return &spec.TypeRef{
			Kind:     spec.KindList,
			ItemType: p.convertTypeRef(tt.Type),
		}
	case *ast.MapType:
		return &spec.TypeRef{
			Kind:      spec.KindMap,
			KeyType:   p.convertTypeRef(tt.KeyType),
			ValueType: p.convertTypeRef(tt.ValueType),
		}
	case *ast.Optional:
		return &spec.TypeRef{
			Kind:         spec.KindOptional,
			OptionalType: p.convertTypeRef(tt.Type),
		}
	case *ast.Stream:
		return &spec.TypeRef{
			Kind:       spec.KindStream,
			StreamType: p.convertTypeRef(tt.Type),
		}
	}

	panic("unreachable")
}

func stringValue(v *ast.StringValue) string {
	if v == nil {
		return ""
	}

	return v.Value
}
