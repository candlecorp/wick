package rest

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"reflect"
	"regexp"
	"sort"
	"strings"

	"github.com/go-logr/logr"
	"github.com/gorilla/handlers"
	"github.com/gorilla/mux"
	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
	"go.opentelemetry.io/otel/trace"

	"github.com/nanobus/nanobus/channel"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/errorz"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/spec"
	"github.com/nanobus/nanobus/transport"
	"github.com/nanobus/nanobus/transport/filter"
	"github.com/nanobus/nanobus/transport/httpresponse"
	"github.com/nanobus/nanobus/transport/middleware"
	"github.com/nanobus/nanobus/transport/routes"
)

type Rest struct {
	log           logr.Logger
	tracer        trace.Tracer
	address       string
	namespaces    spec.Namespaces
	invoker       transport.Invoker
	errorResolver errorz.Resolver
	codecs        map[string]channel.Codec
	middlewares   []middleware.Middleware
	filters       []filter.Filter
	router        *mux.Router
	ln            net.Listener
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
	codecs      []channel.Codec
	middlewares []middleware.Middleware
	filters     []filter.Filter
	routes      []routes.AddRoutes
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

func WithMiddleware(middlewares ...middleware.Middleware) Option {
	return func(opts *optionsHolder) {
		opts.middlewares = middlewares
	}
}

func WithFilters(filters ...filter.Filter) Option {
	return func(opts *optionsHolder) {
		opts.filters = filters
	}
}

func WithRoutes(r ...routes.AddRoutes) Option {
	return func(opts *optionsHolder) {
		opts.routes = r
	}
}

type Configuration struct {
	Address       string        `mapstructure:"address" validate:"required"`
	Static        []StaticPath  `mapstructure:"static"`
	Routes        []Route       `mapstructure:"routes"`
	Documentation Documentation `mapstructure:"documentation"`
}

type Documentation struct {
	SwaggerUI  bool `mapstructure:"swaggerUI"`
	Postman    bool `mapstructure:"postman"`
	RestClient bool `mapstructure:"restClient"`
}

type StaticPath struct {
	Dir   string `mapstructure:"dir" validate:"required"`
	Path  string `mapstructure:"path" validate:"required"`
	Strip string `mapstructure:"strip"`
}

type Route struct {
	Uses string `mapstructure:"uses" validate:"required"`
	With any    `mapstructure:"with"`
}

func Load() (string, transport.Loader) {
	return "rest", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (transport.Transport, error) {
	var jsoncodec channel.Codec
	var msgpackcodec channel.Codec
	var transportInvoker transport.Invoker
	var namespaces spec.Namespaces
	var errorResolver errorz.Resolver
	var middlewares []middleware.Middleware
	var filters []filter.Filter
	var log logr.Logger
	var tracer trace.Tracer
	var routesRegistry routes.Registry
	if err := resolve.Resolve(resolver,
		"codec:json", &jsoncodec,
		"codec:msgpack", &msgpackcodec,
		"transport:invoker", &transportInvoker,
		"spec:namespaces", &namespaces,
		"errors:resolver", &errorResolver,
		"middleware:lookup", &middlewares,
		"filter:lookup", &filters,
		"system:logger", &log,
		"system:tracer", &tracer,
		"registry:routes", &routesRegistry); err != nil {
		return nil, err
	}

	// Defaults
	c := Configuration{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	routes := make([]routes.AddRoutes, len(c.Routes))
	for i, route := range c.Routes {
		r := routesRegistry[route.Uses]
		addRoutes, err := r(ctx, route.With, resolver)
		if err != nil {
			return nil, err
		}
		routes[i] = addRoutes
	}

	return New(log, tracer, c, namespaces, transportInvoker, errorResolver,
		WithMiddleware(middlewares...),
		WithFilters(filters...),
		WithCodecs(jsoncodec, msgpackcodec),
		WithRoutes(routes...))
}

func New(log logr.Logger, tracer trace.Tracer, config Configuration, namespaces spec.Namespaces, invoker transport.Invoker, errorResolver errorz.Resolver, options ...Option) (transport.Transport, error) {
	var opts optionsHolder

	for _, opt := range options {
		opt(&opts)
	}

	codecMap := make(map[string]channel.Codec, len(opts.codecs))
	for _, c := range opts.codecs {
		codecMap[c.ContentType()] = c
	}

	r := mux.NewRouter()
	r.Use(handlers.ProxyHeaders)

	for _, addRoutes := range opts.routes {
		addRoutes(r)
	}

	docsHost := config.Address
	if strings.HasPrefix(docsHost, ":") {
		docsHost = "localhost" + docsHost
	}
	if config.Documentation.SwaggerUI {
		log.Info("Swagger UI", "url", fmt.Sprintf("http://%s/swagger/", docsHost))
		log.Info("Swagger Spec", "url", fmt.Sprintf("http://%s/swagger/swagger_spec", docsHost))
		if err := RegisterSwaggerRoutes(r, namespaces); err != nil {
			return nil, err
		}
	}

	if config.Documentation.Postman {
		log.Info("Postman collection", "url", fmt.Sprintf("http://%s/postman/collection", docsHost))
		if err := RegisterPostmanRoutes(r, namespaces); err != nil {
			return nil, err
		}
	}

	if config.Documentation.RestClient {
		log.Info("VS Code REST Client", "url", fmt.Sprintf("http://%s/rest-client/service.http", docsHost))
		if err := RegisterRESTClientRoutes(r, namespaces); err != nil {
			return nil, err
		}
	}

	rest := Rest{
		log:           log,
		tracer:        tracer,
		address:       config.Address,
		namespaces:    namespaces,
		invoker:       invoker,
		errorResolver: errorResolver,
		codecs:        codecMap,
		middlewares:   opts.middlewares,
		filters:       opts.filters,
		router:        r,
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
									// t := param.Type
									required := true
									isArray := false
									// if t.OptionalType != nil {
									// 	t = t.OptionalType
									// }
									// if t.ItemType != nil {
									// 	t = t.ItemType
									// 	isArray = true
									// }

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
					namespace.Name, service.Name, operation.Name, isActor,
					hasBody, bodyParamName, queryParams)).Methods(methods...)
			}
		}
	}

	sort.Slice(config.Static, func(i, j int) bool {
		return len(config.Static[i].Path) > len(config.Static[j].Path)
	})
	for _, path := range config.Static {
		log.Info("Serving static files",
			"dir", path.Dir,
			"path", path.Path,
			"strip", path.Strip)
		fs := http.FileServer(http.Dir(path.Dir))
		if path.Strip != "" {
			fs = http.StripPrefix(path.Strip, fs)
		}
		r.PathPrefix(path.Path).Handler(handlers.LoggingHandler(os.Stdout, fs))
	}

	return &rest, nil
}

func (t *Rest) Listen() error {
	ln, err := net.Listen("tcp", t.address)
	if err != nil {
		return err
	}
	t.ln = ln
	t.log.Info("REST server listening", "address", t.address)

	// Apply middleware in reverse order.
	var handler http.Handler = t.router
	if len(t.middlewares) > 0 {
		for i := len(t.middlewares) - 1; i >= 0; i-- {
			handler = t.middlewares[i](handler)
		}
	}

	handler = otelhttp.NewHandler(handler, "rest")
	return http.Serve(ln, handler)
}

func (t *Rest) Close() (err error) {
	if t.ln != nil {
		err = t.ln.Close()
		t.ln = nil
	}

	return err
}

func (t *Rest) handler(namespace, service, operation string, isActor bool,
	hasBody bool, bodyParamName string, queryParams map[string]queryParam) func(http.ResponseWriter, *http.Request) {
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

		response, err := t.invoker(ctx, namespace, service, id, operation, input)
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

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
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
