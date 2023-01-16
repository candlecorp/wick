/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package rest

import (
	"bytes"
	"embed"
	"encoding/json"
	"io/fs"
	"net/http"
	"path"
	"sort"
	"strings"

	"github.com/getkin/kin-openapi/openapi3"
	"github.com/gorilla/mux"
	"gopkg.in/yaml.v3"

	"github.com/nanobus/nanobus/pkg/logger"
	"github.com/nanobus/nanobus/pkg/spec"
)

// From https://github.com/flowchartsman/swaggerui
// and adjusted for github.com/gorilla/mux and
// server URL replacement.

//go:generate go run generate_swagger_ui.go

//go:embed embed
var swagfs embed.FS

func RegisterSwaggerRoutes(r *mux.Router, namespaces spec.Namespaces) error {
	specData, err := SpecToOpenAPI3(namespaces)
	if err != nil {
		return err
	}
	r.HandleFunc("/swagger/swagger_spec", swaggerSpecHandler(specData))
	static, err := fs.Sub(swagfs, "embed")
	if err != nil {
		return err
	}
	h := http.StripPrefix("/swagger", http.FileServer(http.FS(static)))
	r.Handle("/swagger/{file}", h)
	r.Handle("/swagger/", h)

	return nil
}

func swaggerSpecHandler(b []byte) http.HandlerFunc {
	return func(w http.ResponseWriter, req *http.Request) {
		var v string
		if req.TLS != nil {
			v = "https://" + req.Host
		} else {
			v = "http://" + req.Host
		}
		swagger := bytes.Replace(b, []byte("[REPLACE_HOST]"), []byte(v), 1)

		if _, err := w.Write(swagger); err != nil {
			logger.Error("could not write swagger bytes", "error", err)
			return
		}
	}
}

func SpecToOpenAPI3(namespaces spec.Namespaces) ([]byte, error) {
	apispec := openapi3.T{
		OpenAPI: "3.0.3",
		Servers: openapi3.Servers{
			&openapi3.Server{
				URL: "[REPLACE_HOST]",
			},
		},
	}

	foundTypes := make(map[string]struct{})

	for _, ns := range namespaces {
		a, ok := ns.Annotation("info")
		if ok {
			var info openapi3.Info
			infoMapJSON, err := json.Marshal(a.ToMap())
			if err != nil {
				return nil, err
			}
			if err := json.Unmarshal(infoMapJSON, &info); err != nil {
				return nil, err
			}
			apispec.Info = &info
		}

		nsPath := getAnotationString(ns, "path")
		for _, service := range ns.Services {
			_, isService := service.Annotation("service")
			_, isActor := service.Annotation("actor")
			_, isStateful := service.Annotation("stateful")
			_, isWorkflow := service.Annotation("workflow")

			isActor = isActor || isStateful || isWorkflow
			if !(isService || isActor) {
				continue
			}

			servicePath := getAnotationString(service, "path")

			addTag := false

			for _, oper := range service.Operations {
				operPath := getAnotationString(oper, "path")
				p := path.Clean(path.Join(nsPath, servicePath, operPath))

				if apispec.Paths == nil {
					apispec.Paths = make(openapi3.Paths)
				}
				sp, existingPathItem := apispec.Paths[p]
				if !existingPathItem {
					sp = &openapi3.PathItem{}
				}

				var operPtr **openapi3.Operation
				if _, ok := oper.Annotation("GET"); ok {
					operPtr = &sp.Get
				} else if _, ok := oper.Annotation("OPTIONS"); ok {
					operPtr = &sp.Head
				} else if _, ok := oper.Annotation("HEAD"); ok {
					operPtr = &sp.Options
				} else if _, ok := oper.Annotation("PATCH"); ok {
					operPtr = &sp.Patch
				} else if _, ok := oper.Annotation("POST"); ok {
					operPtr = &sp.Post
				} else if _, ok := oper.Annotation("PUT"); ok {
					operPtr = &sp.Put
				} else if _, ok := oper.Annotation("DELETE"); ok {
					operPtr = &sp.Delete
				} else {
					continue
				}

				if oper.Returns != nil {
					traverseTypeRef(foundTypes, oper.Returns)
				}

				if oper.Parameters != nil {
					if oper.Unary {
						traverseType(foundTypes, oper.Parameters)
					} else {
						for _, p := range oper.Parameters.Fields {
							_, hasQuery := p.Annotation("query")
							if !hasQuery && p.Type.Type != nil {
								traverseType(foundTypes, p.Type.Type)
							}
						}
					}
				}

				if !existingPathItem {
					apispec.Paths[p] = sp
				}

				var responses openapi3.Responses
				if oper.Returns != nil {
					switch oper.Returns.Kind {
					case spec.KindType:
						responses = openapi3.NewResponses()
						defaultResponse := responses.Default()
						defaultResponse.Value.
							WithDescription("Success").
							WithJSONSchemaRef(
								openapi3.NewSchemaRef(
									"#/components/schemas/"+oper.Returns.Type.Name, nil))
					case spec.KindList:
						responses = openapi3.NewResponses()
						defaultResponse := responses.Default()
						ary := openapi3.NewArraySchema()
						ary.Items = openapi3.NewSchemaRef(
							"#/components/schemas/"+oper.Returns.ItemType.Type.Name, nil)
						defaultResponse.Value.
							WithDescription("Success").
							WithJSONSchemaRef(ary.NewRef())
					default:
						primitive := typeFormat(oper.Returns)
						if primitive != nil {
							responses = openapi3.NewResponses()
							defaultResponse := responses.Default()
							defaultResponse.Value.WithJSONSchema(primitive)
						}
					}
				} else {
					responses = openapi3.NewResponses()
					responses["204"] = &openapi3.ResponseRef{
						Value: openapi3.NewResponse().WithDescription(("Success")),
					}
				}

				params, rb := parameters(p, service, oper)
				o := openapi3.Operation{
					Tags:        []string{service.Name},
					Summary:     getAnotationString(oper, "summary"),
					Description: oper.Description,
					OperationID: oper.Name,
					Parameters:  params,
					RequestBody: rb,
					Responses:   responses,
				}

				*operPtr = &o

				addTag = true
			}

			if addTag {
				// Add tag
				apispec.Tags = append(apispec.Tags, &openapi3.Tag{
					Name:        service.Name,
					Description: service.Description,
				})
			}
		}

		if len(apispec.Components.Schemas) == 0 && len(ns.Types) > 0 {
			apispec.Components.Schemas = openapi3.Schemas{}
		}

		sortedTypeNames := make([]string, 0, len(foundTypes))
		for name := range foundTypes {
			sortedTypeNames = append(sortedTypeNames, name)
		}
		sort.Strings(sortedTypeNames)

		for _, name := range sortedTypeNames {
			t, _ := ns.Type(name)
			if t == nil {
				continue
			}
			apispec.Components.Schemas[t.Name] = &openapi3.SchemaRef{
				Value: &openapi3.Schema{
					Description: t.Description,
					Properties:  properties(t.Fields),
				},
			}
		}

		// TODO: Enums, Unions
	}

	specBytes, err := apispec.MarshalJSON()
	if err != nil {
		return nil, err
	}

	var specData interface{}
	if err := json.Unmarshal(specBytes, &specData); err != nil {
		logger.Warn("failed to decode spec bytes", "error", err)
	}
	specBytesIndented, _ := yaml.Marshal(specData)

	return specBytesIndented, nil
}

func properties(fields []*spec.Field) map[string]*openapi3.SchemaRef {
	props := make(map[string]*openapi3.SchemaRef, len(fields))

	for _, f := range fields {
		if f.Type.Kind == spec.KindType {
			props[f.Name] = &openapi3.SchemaRef{
				Ref: "#/components/schemas/" + f.Type.Type.Name,
			}
		} else {
			props[f.Name] = &openapi3.SchemaRef{
				Value: fieldToValue(f),
			}
		}
	}

	return props
}

func parameters(path string, service *spec.Service, oper *spec.Operation) (openapi3.Parameters, *openapi3.RequestBodyRef) {
	params := make(openapi3.Parameters, 0, len(oper.Parameters.Fields)+1)
	var requestBody *openapi3.RequestBodyRef

	pathParams := map[string]struct{}{}
	for _, match := range rePathParams.FindAllString(path, -1) {
		match = strings.TrimPrefix(match, "{")
		match = strings.TrimSuffix(match, "}")
		pathParams[match] = struct{}{}
	}

	_, isActor := service.Annotation("actor")
	_, isStateful := service.Annotation("stateful")
	_, isWorkflow := service.Annotation("workflow")

	isActor = isActor || isStateful || isWorkflow

	if isActor {
		params = append(params, &openapi3.ParameterRef{
			Value: openapi3.NewPathParameter("id").
				WithDescription("State identifier").
				WithSchema(openapi3.NewStringSchema()),
		})
	}

	if !oper.Unary {
		bodyParams := []*spec.Field{}
		for _, param := range oper.Parameters.Fields {
			// Look for path parameters by name.
			if _, ok := pathParams[param.Name]; ok {
				params = append(params, &openapi3.ParameterRef{
					Value: openapi3.NewPathParameter(param.Name).
						WithDescription(param.Description).
						WithSchema(fieldToValue(param)),
				})
			} else if _, ok := param.Annotation("query"); ok {
				// Handle query parameters
				if param.Type.IsPrimitive() {
					params = append(params, &openapi3.ParameterRef{
						Value: openapi3.NewQueryParameter(param.Name).
							WithDescription(param.Description).
							WithSchema(fieldToValue(param)),
					})
				} else if param.Type.Type != nil {
					for _, f := range param.Type.Type.Fields {
						params = append(params, &openapi3.ParameterRef{
							Value: openapi3.NewQueryParameter(f.Name).
								WithDescription(f.Description).
								WithSchema(fieldToValue(f)),
						})
					}
				}
			} else {
				bodyParams = append(bodyParams, param)
			}
		}
		if len(bodyParams) > 0 {
			body := openapi3.NewRequestBody().WithSchema(
				&openapi3.Schema{
					Properties: properties(bodyParams),
				},
				[]string{"application/json"},
			)
			requestBody = &openapi3.RequestBodyRef{
				Value: body,
			}
		}
	} else {
		param := oper.Parameters
		if _, ok := param.Annotation("query"); ok {
			for _, f := range param.Fields {
				params = append(params, &openapi3.ParameterRef{
					Value: openapi3.NewQueryParameter(f.Name).
						WithDescription(f.Description).
						WithSchema(typeFormat(f.Type)),
				})
			}
		} else {
			requestBody = &openapi3.RequestBodyRef{
				Value: openapi3.NewRequestBody().
					WithRequired(true).
					WithDescription(param.Description).
					WithJSONSchemaRef(openapi3.NewSchemaRef("#/components/schemas/"+param.Name, nil)),
			}
		}
	}

	if len(params) == 0 {
		params = nil
	}

	return params, requestBody
}

func traverseTypeRef(foundTypes map[string]struct{}, t *spec.TypeRef) {
	switch t.Kind {
	case spec.KindType:
		if !t.IsPrimitive() {
			traverseType(foundTypes, t.Type)
		}
	case spec.KindMap:
		traverseTypeRef(foundTypes, t.KeyType)
		traverseTypeRef(foundTypes, t.ValueType)
	case spec.KindList:
		traverseTypeRef(foundTypes, t.ItemType)
	case spec.KindOptional:
		traverseTypeRef(foundTypes, t.OptionalType)
	}
}

func traverseType(foundTypes map[string]struct{}, t *spec.Type) {
	if t == nil {
		return
	}
	if t.Name != "" {
		foundTypes[t.Name] = struct{}{}
	}
	for _, f := range t.Fields {
		traverseTypeRef(foundTypes, f.Type)
	}
}

func fieldToValue(f *spec.Field) *openapi3.Schema {
	t := typeFormat(f.Type)
	t.Description = f.Description
	t.Default = f.DefaultValue
	return t
}

func typeFormat(t *spec.TypeRef) *openapi3.Schema {
	if t == nil {
		return nil
	}

	switch t.Kind {
	case spec.KindString:
		return openapi3.NewStringSchema()
	case spec.KindBool:
		return openapi3.NewBoolSchema()
	case spec.KindU64, spec.KindI64:
		return openapi3.NewInt64Schema()
	case spec.KindU32, spec.KindI32, spec.KindU16, spec.KindI16, spec.KindU8, spec.KindI8:
		return openapi3.NewInt32Schema()
	case spec.KindBytes:
		return openapi3.NewBytesSchema()
	case spec.KindF64, spec.KindF32:
		return openapi3.NewFloat64Schema()
	case spec.KindRaw:
		return openapi3.NewObjectSchema()
		// TODO: Any
	case spec.KindDateTime:
		return openapi3.NewDateTimeSchema()
	case spec.KindList:
		if t.ItemType.Kind == spec.KindType {
			return &openapi3.Schema{
				Type: "array",
				Items: &openapi3.SchemaRef{
					Ref: "#/components/schemas/" + t.ItemType.Type.Name,
				},
			}
		}
		return openapi3.NewArraySchema().
			WithItems(typeFormat(t.ItemType))
	case spec.KindMap:
		if t.ValueType.Kind == spec.KindType {
			return &openapi3.Schema{
				Type: "object",
				AdditionalProperties: openapi3.AdditionalProperties{
					Schema: &openapi3.SchemaRef{
						Ref: "#/components/schemas/" + t.ValueType.Type.Name,
					},
				},
			}
		}
		return openapi3.NewObjectSchema().
			WithAdditionalProperties(typeFormat(t.ValueType))
	case spec.KindOptional:
		return typeFormat(t.OptionalType)
	case spec.KindEnum:
		values := make([]interface{}, len(t.Enum.Values))
		for i, v := range t.Enum.Values {
			values[i] = v.StringValue
		}
		return &openapi3.Schema{
			Type: "string",
			Enum: values,
		}
	}

	return nil
}

func getAnotationString(a spec.Annotator, name string) string {
	if a, ok := a.Annotation(name); ok {
		if arg, ok := a.Argument("value"); ok {
			return arg.ValueString()
		}
	}
	return ""
}
