/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package rest

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"reflect"
	"regexp"
	"strings"

	"github.com/go-logr/logr"
	"github.com/gorilla/mux"
	"go.opentelemetry.io/otel/trace"

	"github.com/nanobus/nanobus/pkg/channel"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/spec"
	"github.com/nanobus/nanobus/pkg/transport"
	"github.com/nanobus/nanobus/pkg/transport/filter"
	"github.com/nanobus/nanobus/pkg/transport/http/router"
	"github.com/nanobus/nanobus/pkg/transport/httpresponse"
)

type Rest struct {
	log           logr.Logger
	tracer        trace.Tracer
	namespaces    spec.Namespaces
	invoker       transport.Invoker
	errorResolver errorz.Resolver
	codecs        map[string]channel.Codec
	filters       []filter.Filter
}

type queryParam struct {
	name         string
	arg          string
	isArray      bool
	required     bool
	typeRef      *spec.TypeRef
	defaultValue interface{}
}

type optionsHolder struct {
	codecs  []channel.Codec
	filters []filter.Filter
}

var rePathParams = regexp.MustCompile(`(?m)\{([^\}]*)\}`)

var (
	ErrUnregisteredContentType = errors.New("unregistered content type")
	ErrInvalidURISyntax        = errors.New("invalid invocation URI syntax")
)

type Option func(opts *optionsHolder)

func WithCodecs(codecs ...channel.Codec) Option {
	return func(opts *optionsHolder) {
		opts.codecs = codecs
	}
}

func WithFilters(filters ...filter.Filter) Option {
	return func(opts *optionsHolder) {
		opts.filters = filters
	}
}

func RestV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (router.Router, error) {
	var jsoncodec channel.Codec
	var msgpackcodec channel.Codec
	var transportInvoker transport.Invoker
	var namespaces spec.Namespaces
	var errorResolver errorz.Resolver
	var filters []filter.Filter
	var log logr.Logger
	var tracer trace.Tracer
	if err := resolve.Resolve(resolver,
		"codec:json", &jsoncodec,
		"codec:msgpack", &msgpackcodec,
		"transport:invoker", &transportInvoker,
		"spec:namespaces", &namespaces,
		"errors:resolver", &errorResolver,
		"filter:lookup", &filters,
		"system:logger", &log,
		"system:tracer", &tracer); err != nil {
		return nil, err
	}

	// Defaults
	c := RestV1Config{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return NewV1(log, tracer, c, namespaces, transportInvoker, errorResolver,
		WithCodecs(jsoncodec, msgpackcodec), WithFilters(filters...))
}

func NewV1(log logr.Logger, tracer trace.Tracer, config RestV1Config, namespaces spec.Namespaces, invoker transport.Invoker, errorResolver errorz.Resolver, options ...Option) (router.Router, error) {
	var opts optionsHolder

	for _, opt := range options {
		opt(&opts)
	}

	codecMap := make(map[string]channel.Codec, len(opts.codecs))
	for _, c := range opts.codecs {
		codecMap[c.ContentType()] = c
	}

	return func(r *mux.Router, address string) error {
		rest := Rest{
			log:           log,
			tracer:        tracer,
			namespaces:    namespaces,
			invoker:       invoker,
			errorResolver: errorResolver,
			codecs:        codecMap,
			filters:       opts.filters,
		}

		docsHost := address
		if strings.HasPrefix(docsHost, ":") {
			docsHost = "localhost" + docsHost
		}
		if config.Documentation.SwaggerUI != nil && *config.Documentation.SwaggerUI {
			log.Info("Swagger UI", "url", fmt.Sprintf("http://%s/swagger/", docsHost))
			log.Info("Swagger Spec", "url", fmt.Sprintf("http://%s/swagger/swagger_spec", docsHost))
			if err := RegisterSwaggerRoutes(r, namespaces); err != nil {
				return err
			}
		}

		if config.Documentation.Postman != nil && *config.Documentation.Postman {
			log.Info("Postman collection", "url", fmt.Sprintf("http://%s/postman/collection", docsHost))
			if err := RegisterPostmanRoutes(r, namespaces); err != nil {
				return err
			}
		}

		if config.Documentation.RestClient != nil && *config.Documentation.RestClient {
			log.Info("VS Code REST Client", "url", fmt.Sprintf("http://%s/rest-client/service.http", docsHost))
			if err := RegisterRESTClientRoutes(r, namespaces); err != nil {
				return err
			}
		}

		for _, namespace := range namespaces {
			pathNS := ""
			if path, ok := namespace.Annotation("path"); ok {
				if arg, ok := path.Argument("value"); ok {
					pathNS = arg.ValueString()
				}
			}

			for _, service := range namespace.Services {
				_, isService := service.Annotation("service")
				_, isActor := service.Annotation("actor")
				_, isStateful := service.Annotation("stateful")
				_, isWorkflow := service.Annotation("workflow")
				isActor = isActor || isStateful || isWorkflow

				if !(isService || isActor) {
					continue
				}

				pathSrv := ""
				if path, ok := service.Annotation("path"); ok {
					if arg, ok := path.Argument("value"); ok {
						pathSrv = arg.ValueString()
					}
				}

				for _, operation := range service.Operations {
					pathOper := ""
					if path, ok := operation.Annotation("path"); ok {
						if arg, ok := path.Argument("value"); ok {
							pathOper = arg.ValueString()
						}
					}

					path := pathNS + pathSrv + pathOper
					if path == "" {
						continue
					}

					methods := []string{}
					if _, ok := operation.Annotation("GET"); ok {
						methods = append(methods, "GET")
					}
					if _, ok := operation.Annotation("POST"); ok {
						methods = append(methods, "POST")
					}
					if _, ok := operation.Annotation("PUT"); ok {
						methods = append(methods, "PUT")
					}
					if _, ok := operation.Annotation("PATCH"); ok {
						methods = append(methods, "PATCH")
					}
					if _, ok := operation.Annotation("DELETE"); ok {
						methods = append(methods, "DELETE")
					}
					if len(methods) == 0 {
						continue
					}

					bodyParamName := ""
					hasBody := false
					queryParams := map[string]queryParam{}

					if !operation.Unary {
						pathParams := map[string]struct{}{}
						for _, match := range rePathParams.FindAllString(path, -1) {
							match = strings.TrimPrefix(match, "{")
							match = strings.TrimSuffix(match, "}")
							pathParams[match] = struct{}{}
						}

						for _, param := range operation.Parameters.Fields {
							if _, ok := pathParams[param.Name]; ok {
								continue
							} else if _, ok := param.Annotation("query"); ok {
								t := param.Type
								required := true
								isArray := false
								if t.OptionalType != nil {
									required = false
									t = t.OptionalType
								}
								if t.ItemType != nil {
									t = t.ItemType
									isArray = true
								}
								if t.IsPrimitive() {
									queryParams[param.Name] = queryParam{
										name:         param.Name,
										required:     required,
										isArray:      isArray,
										typeRef:      t,
										defaultValue: param.DefaultValue,
									}
								} else if t.Type != nil {
									for _, f := range t.Type.Fields {
										required := true
										isArray := false

										queryParams[f.Name] = queryParam{
											name:         f.Name,
											arg:          param.Name,
											required:     required,
											isArray:      isArray,
											typeRef:      f.Type,
											defaultValue: f.DefaultValue,
										}
									}
								}
							} else if _, ok := param.Annotation("body"); ok {
								bodyParamName = param.Name
								hasBody = true
							}
						}
					} else {
						_, hasQuery := operation.Parameters.Annotation("query")
						if hasQuery {
							for _, param := range operation.Parameters.Fields {
								if param.Type.IsPrimitive() {
									queryParams[param.Name] = queryParam{
										name:         param.Name,
										isArray:      false, // TODO
										typeRef:      param.Type,
										defaultValue: param.DefaultValue,
									}
								} else {
									for _, f := range param.Type.Type.Fields {
										queryParams[f.Name] = queryParam{
											name:         f.Name,
											isArray:      false, // TODO
											typeRef:      f.Type,
											defaultValue: f.DefaultValue,
										}
									}
								}
							}
						} else {
							hasBody = true
						}
					}

					log.Info("Registering REST handler", "methods", methods, "path", path)
					r.HandleFunc(path, rest.handler(
						namespace.Name+"."+service.Name, operation.Name, isActor,
						hasBody, bodyParamName, queryParams)).Methods(methods...)
				}
			}
		}

		return nil
	}, nil
}

func (t *Rest) handler(iface, operation string, isActor bool,
	hasBody bool, bodyParamName string, queryParams map[string]queryParam) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		defer r.Body.Close()
		vars := mux.Vars(r)
		id := ""
		if isActor {
			id = vars["id"]
		}

		contentType := r.Header.Get("Content-Type")
		if contentType == "" {
			contentType = "application/json"
		}

		codec, ok := t.codecs[contentType]
		if !ok {
			w.WriteHeader(http.StatusUnsupportedMediaType)
			fmt.Fprintf(w, "%v: %s", ErrUnregisteredContentType, contentType)
			return
		}

		resp := httpresponse.New()
		ctx = httpresponse.NewContext(ctx, resp)

		for _, filter := range t.filters {
			var err error
			if ctx, err = filter(ctx, r.Header); err != nil {
				t.handleError(err, codec, r, w, http.StatusInternalServerError)
				return
			}
		}

		requestBytes, err := io.ReadAll(r.Body)
		if err != nil {
			t.handleError(err, codec, r, w, http.StatusInternalServerError)
			return
		}

		var input map[string]interface{}
		if len(requestBytes) > 0 {
			if bodyParamName == "" {
				if err := codec.Decode(requestBytes, &input); err != nil {
					t.handleError(err, codec, r, w, http.StatusInternalServerError)
					return
				}
			} else {
				var body interface{}
				if err := codec.Decode(requestBytes, &body); err != nil {
					t.handleError(err, codec, r, w, http.StatusInternalServerError)
					return
				}
				input = map[string]interface{}{
					bodyParamName: body,
				}
			}
		} else {
			input = make(map[string]interface{}, len(vars)+len(queryParams))
		}

		for name, value := range vars {
			input[name] = value
		}

		if len(queryParams) > 0 {
			queryValues, _ := url.ParseQuery(r.URL.RawQuery)
			for name, q := range queryParams {
				if values, ok := queryValues[name]; ok {
					var converted interface{}
					if q.isArray {
						items := make([]interface{}, 0, 100)
						for _, value := range values {
							parts := strings.Split(value, ",")
							for _, v := range parts {
								converted, _, err = q.typeRef.Coalesce(v, false)
								if err != nil {
									t.handleError(err, codec, r, w, http.StatusBadRequest)
									return
								}
								items = append(items, converted)
							}
						}
						converted = items
					} else {
						converted, _, err = q.typeRef.Coalesce(values[0], false)
						if err != nil {
							t.handleError(err, codec, r, w, http.StatusBadRequest)
							return
						}
					}
					if converted == nil {
						converted = q.defaultValue
					}
					wrapper := input
					if q.arg != "" {
						var w interface{}
						found := false
						if w, found = input[q.arg]; found {
							wrapper, found = w.(map[string]interface{})
						}
						if !found {
							wrapper = make(map[string]interface{}, len(queryValues))
							input[q.arg] = wrapper
						}
					}
					wrapper[name] = converted
				} else if q.isArray && q.required {
					input[name] = []interface{}{}
				} else if q.defaultValue != nil {
					wrapper := input
					if q.arg != "" {
						var w interface{}
						found := false
						if w, found = input[q.arg]; found {
							wrapper, found = w.(map[string]interface{})
						}
						if !found {
							wrapper = make(map[string]interface{}, len(queryValues))
							input[q.arg] = wrapper
						}
					}
					wrapper[name] = q.defaultValue
				}
			}
		}

		response, err := t.invoker(ctx, iface, id, operation, input, transport.PerformAuthorization)
		if err != nil {
			code := http.StatusInternalServerError
			if errors.Is(err, transport.ErrBadInput) {
				code = http.StatusBadRequest
			}
			t.handleError(err, codec, r, w, code)
			return
		}

		if !isNil(response) {
			header := w.Header()
			header.Set("Content-Type", codec.ContentType())
			for k, vals := range resp.Header {
				for _, v := range vals {
					header.Add(k, v)
				}
			}
			w.WriteHeader(resp.Status)
			responseBytes, err := codec.Encode(response)
			if err != nil {
				t.handleError(err, codec, r, w, http.StatusInternalServerError)
				return
			}

			w.Write(responseBytes)
		} else {
			w.WriteHeader(http.StatusNoContent)
		}
	}
}

func (t *Rest) handleError(err error, codec channel.Codec, req *http.Request, w http.ResponseWriter, status int) {
	var errz *errorz.Error
	if !errors.As(err, &errz) {
		errz = t.errorResolver(err)
	}
	errz.Path = req.RequestURI

	w.Header().Add("Content-Type", codec.ContentType())
	w.WriteHeader(errz.Status)
	payload, err := codec.Encode(errz)
	if err != nil {
		fmt.Fprint(w, "unknown error")
	}

	w.Write(payload)
}

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
}
