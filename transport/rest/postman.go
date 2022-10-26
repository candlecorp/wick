package rest

import (
	"bytes"
	"encoding/json"
	"net/http"
	"path"
	"strings"
	"time"

	"github.com/getkin/kin-openapi/openapi3"
	"github.com/gorilla/mux"
	postman "github.com/rbretecher/go-postman-collection"

	"github.com/nanobus/nanobus/spec"
)

func RegisterPostmanRoutes(r *mux.Router, namespaces spec.Namespaces) error {
	specData, err := SpecToPostmanCollection(namespaces)
	if err != nil {
		return err
	}
	r.HandleFunc("/postman/collection", func(w http.ResponseWriter, req *http.Request) {
		var v string
		if req.TLS != nil {
			v = "https://" + req.Host
		} else {
			v = "http://" + req.Host
		}
		replaced := bytes.Replace(specData, []byte("[REPLACE_HOST]"), []byte(v), 1)
		w.Write(replaced)
	})

	return nil
}

func SpecToPostmanCollection(namespaces spec.Namespaces) ([]byte, error) {
	c := postman.Collection{
		Info: postman.Info{
			Schema: "https://schema.getpostman.com/json/collection/v2.1.0/collection.json",
		},
		Items: []*postman.Items{},
		Variables: []*postman.Variable{
			{
				Key:   "baseRestURI",
				Value: "[REPLACE_HOST]",
				Type:  "string",
			},
		},
	}

	for _, ns := range namespaces {
		c.Info.Name = ns.Name
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
			c.Info.Name = info.Title
			if len(info.Description) > 0 {
				c.Info.Description = postman.Description{
					Content: info.Description,
				}
			}
		}

		nsPath := getAnotationString(ns, "path")
		for _, service := range ns.Services {
			serviceItem := postman.Items{
				Name:        service.Name,
				Description: service.Description,
				Items:       []*postman.Items{},
			}
			_, isService := service.Annotation("service")
			_, isActor := service.Annotation("actor")
			_, isStateful := service.Annotation("stateful")
			_, isWorkflow := service.Annotation("workflow")

			isActor = isActor || isStateful || isWorkflow
			if !(isService || isActor) {
				continue
			}

			servicePath := getAnotationString(service, "path")

			for _, oper := range service.Operations {
				operPath := getAnotationString(oper, "path")
				p := path.Clean(path.Join(nsPath, servicePath, operPath))

				var method string
				if _, ok := oper.Annotation("GET"); ok {
					method = "GET"
				} else if _, ok := oper.Annotation("OPTIONS"); ok {
					method = "OPTIONS"
				} else if _, ok := oper.Annotation("HEAD"); ok {
					method = "HEAD"
				} else if _, ok := oper.Annotation("PATCH"); ok {
					method = "PATCH"
				} else if _, ok := oper.Annotation("POST"); ok {
					method = "POST"
				} else if _, ok := oper.Annotation("PUT"); ok {
					method = "PUT"
				} else if _, ok := oper.Annotation("DELETE"); ok {
					method = "DELETE"
				} else {
					continue
				}

				operItem := postman.Items{
					Name:        oper.Name,
					Description: oper.Description,
					Request: &postman.Request{
						Description: oper.Description, // Allows GET operations to import correctly.
						URL: &postman.URL{
							Raw:  "{{baseRestURI}}" + p,
							Host: []string{"{{baseRestURI}}"},
							Path: strings.Split(strings.TrimPrefix(p, "/"), "/"),
						},
						Method: postman.Method(method),
					},
				}

				if len(oper.Parameters.Fields) > 0 {
					_, raw := exampleOperationRequestBody(p, service, oper, 4)
					operItem.Request.Body = &postman.Body{
						Mode: "raw",
						Raw:  raw,
						Options: &postman.BodyOptions{
							Raw: postman.BodyOptionsRaw{
								Language: "json",
							},
						},
					}
				}

				serviceItem.Items = append(serviceItem.Items, &operItem)
			}

			c.Items = append(c.Items, &serviceItem)
		}
	}

	return json.MarshalIndent(c, "", "  ")
}

func exampleOperationRequestBody(path string, service *spec.Service, oper *spec.Operation, indent int) ([]string, string) {
	pathParams := []string{}
	pathParamMap := map[string]struct{}{}
	for _, match := range rePathParams.FindAllString(path, -1) {
		match = strings.TrimPrefix(match, "{")
		match = strings.TrimSuffix(match, "}")
		pathParamMap[match] = struct{}{}
		pathParams = append(pathParams, match)
	}

	bodyParams := []*spec.Field{}
	for _, param := range oper.Parameters.Fields {
		_, isPath := pathParamMap[param.Name]
		_, isQuery := param.Annotation("query")
		if !isPath && !isQuery {
			bodyParams = append(bodyParams, param)
		}
	}
	examplePayload := map[string]interface{}{}
	for _, f := range bodyParams {
		examplePayload[f.Name] = exampleValue(f.Annotated, f.Type)
	}

	if len(examplePayload) == 0 {
		return pathParams, ""
	}

	exampleBytes, err := json.MarshalIndent(examplePayload, "", strings.Repeat(" ", indent))
	if err != nil {
		return pathParams, ""
	}

	return pathParams, string(exampleBytes)
}

func exampleValue(a spec.Annotated, t *spec.TypeRef) interface{} {
	example, ok := a.Annotation("example")
	if ok && len(example.Arguments) > 0 {
		return example.Arguments[0].Value
	}

	switch t.Kind {
	case spec.KindString:
		return "string"
	case spec.KindBool:
		return false
	case spec.KindU64, spec.KindI64:
		return 0
	case spec.KindU32, spec.KindI32, spec.KindU16, spec.KindI16, spec.KindU8, spec.KindI8:
		return 0
	case spec.KindBytes:
		return []byte{}
	case spec.KindF64, spec.KindF32:
		return 0.0
	case spec.KindRaw:
		return map[string]interface{}{}
	case spec.KindDateTime:
		return time.Now().Format(time.RFC3339Nano)
	case spec.KindList:
		return []interface{}{exampleValue(spec.Annotated{}, t.ItemType)}
	case spec.KindMap:
		return map[string]interface{}{
			"key": exampleValue(spec.Annotated{}, t.ValueType),
		}
	case spec.KindOptional:
		return exampleValue(spec.Annotated{}, t.OptionalType)
	case spec.KindEnum:
		if len(t.Enum.Values) > 0 {
			return t.Enum.Values[0].StringValue
		}
	}

	return nil
}
