/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package engine

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"syscall"
	"time"

	"github.com/go-logr/logr"
	"github.com/joho/godotenv"
	"go.uber.org/zap/zapcore"

	"github.com/oklog/run"
	"github.com/vmihailenco/msgpack/v5"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/propagation"
	otel_resource "go.opentelemetry.io/otel/sdk/resource"
	sdk_trace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.12.0"
	"go.opentelemetry.io/otel/trace"

	// COMPONENT MODEL / PLUGGABLE COMPONENTS

	// NANOBUS CORE
	"github.com/nanobus/nanobus/pkg/channel"
	"github.com/nanobus/nanobus/pkg/coalesce"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/initialize"
	"github.com/nanobus/nanobus/pkg/logger"
	"github.com/nanobus/nanobus/pkg/mesh"
	"github.com/nanobus/nanobus/pkg/oci"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/runtime"
	"github.com/nanobus/nanobus/pkg/security/authorization"
	"github.com/nanobus/nanobus/pkg/security/claims"

	// CHANNELS
	bytes_codec "github.com/nanobus/nanobus/pkg/channel/codecs/bytes"
	json_codec "github.com/nanobus/nanobus/pkg/channel/codecs/json"
	msgpack_codec "github.com/nanobus/nanobus/pkg/channel/codecs/msgpack"

	// SPECIFICATION LANGUAGES
	"github.com/nanobus/nanobus/pkg/spec"
	spec_apex "github.com/nanobus/nanobus/pkg/spec/apex"

	// COMPONENTS
	"github.com/nanobus/iota/go/payload"
	"github.com/nanobus/nanobus/pkg/compute"
	compute_wasmrs "github.com/nanobus/nanobus/pkg/compute/wasmrs"

	// ACTIONS
	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/blob"
	"github.com/nanobus/nanobus/pkg/actions/core"
	"github.com/nanobus/nanobus/pkg/actions/dapr"
	"github.com/nanobus/nanobus/pkg/actions/postgres"

	// CODECS
	"github.com/nanobus/nanobus/pkg/codec"
	codec_bytes "github.com/nanobus/nanobus/pkg/codec/bytes"
	cloudevents_avro "github.com/nanobus/nanobus/pkg/codec/cloudevents/avro"
	cloudevents_json "github.com/nanobus/nanobus/pkg/codec/cloudevents/json"
	"github.com/nanobus/nanobus/pkg/codec/confluentavro"
	codec_json "github.com/nanobus/nanobus/pkg/codec/json"
	codec_msgpack "github.com/nanobus/nanobus/pkg/codec/msgpack"
	codec_text "github.com/nanobus/nanobus/pkg/codec/text"

	// INITIALIZERS / DB MIGRATION
	migrate_postgres "github.com/nanobus/nanobus/pkg/initialize/postgres"

	// TELEMETRY / TRACING
	otel_tracing "github.com/nanobus/nanobus/pkg/telemetry/tracing"
	tracing_jaeger "github.com/nanobus/nanobus/pkg/telemetry/tracing/jaeger"
	tracing_otlp "github.com/nanobus/nanobus/pkg/telemetry/tracing/otlp"
	tracing_stdout "github.com/nanobus/nanobus/pkg/telemetry/tracing/stdout"

	// TRANSPORTS
	"github.com/nanobus/nanobus/pkg/transport"
	transport_dapr "github.com/nanobus/nanobus/pkg/transport/dapr"
	transport_http "github.com/nanobus/nanobus/pkg/transport/http"
	transport_httprpc "github.com/nanobus/nanobus/pkg/transport/httprpc"
	transport_nats "github.com/nanobus/nanobus/pkg/transport/nats"
	transport_time "github.com/nanobus/nanobus/pkg/transport/time"

	// TRANSPORT - FILTERS
	"github.com/nanobus/nanobus/pkg/transport/filter"
	"github.com/nanobus/nanobus/pkg/transport/filter/jwt"
	"github.com/nanobus/nanobus/pkg/transport/filter/paseto"
	"github.com/nanobus/nanobus/pkg/transport/filter/session"
	"github.com/nanobus/nanobus/pkg/transport/filter/userinfo"

	// TRANSPORT - HTTP MIDDLEWARE
	"github.com/nanobus/nanobus/pkg/transport/http/middleware"
	middleware_cors "github.com/nanobus/nanobus/pkg/transport/http/middleware/cors"

	// TRANSPORT - HTTP ROUTERS
	"github.com/nanobus/nanobus/pkg/transport/http/router"
	router_oauth2 "github.com/nanobus/nanobus/pkg/transport/http/router/oauth2"
	router_rest "github.com/nanobus/nanobus/pkg/transport/http/router/rest"
	router_router "github.com/nanobus/nanobus/pkg/transport/http/router/router"
	router_static "github.com/nanobus/nanobus/pkg/transport/http/router/static"
)

// type Runtime struct {
// 	log        logr.Logger
// 	config     *runtime.BusConfig
// 	namespaces spec.Namespaces
// 	processor  runtime.Namespaces
// 	resolver   resolve.DependencyResolver
// 	resolveAs  resolve.ResolveAs
// 	env        runtime.Environment
// }

type Mode int

const (
	ModeService Mode = iota
	ModeInvoke
)

type Info struct {
	Mode          Mode
	Target        string
	ResourcesFile string
	DeveloperMode bool
	LogLevel      zapcore.Level

	// Service mode
	Process []string

	// CLI mode
	EntityID string
}

type Engine struct {
	ctx            context.Context
	log            logr.Logger
	tracer         trace.Tracer
	actionRegistry actions.Registry
	resolver       resolve.DependencyResolver
	resolveAs      resolve.ResolveAs
	namespaces     spec.Namespaces
	m              *mesh.Mesh
	allNamespaces  runtime.Namespaces
	codec          channel.Codec

	transportInvoker transport.Invoker
}

func (e *Engine) LoadConfig(busConfig *runtime.BusConfig) error {
	// Create processor
	processor, err := runtime.NewProcessor(e.ctx, e.log, e.tracer, e.actionRegistry, e.resolver)
	if err != nil {
		e.log.Error(err, "Could not create NanoBus runtime")
		return err
	}

	if busConfig.Spec != nil {
		specFile, err := config.NormalizeUrl(*busConfig.Spec, *busConfig.BaseURL)
		if err != nil {
			e.log.Error(err, "Error parsing spec into URI", "input", busConfig.Spec)
			return err
		}
		e.log.Info("Loading interface specification", "filename", specFile)
		interfaceExt := filepath.Ext(specFile)
		var nss []*spec.Namespace
		switch interfaceExt {
		case ".apex", ".axdl", ".aidl", ".apexlang":
			nss, err = spec_apex.Loader(e.ctx, map[string]interface{}{
				"filename": specFile,
			}, e.resolveAs)
		default:
			e.log.Error(err, "Unknown spec type", "filename", specFile)
			return err
		}
		if err != nil {
			e.log.Error(err, "Error loading spec", "filename", specFile)
			return err
		}
		for _, ns := range nss {
			e.namespaces[ns.Name] = ns
		}
	}

	if busConfig.Main != nil {
		file, err := config.NormalizeUrl(*busConfig.Main, *busConfig.BaseURL)
		if err != nil {
			e.log.Error(err, "Error parsing spec into URI", "input", busConfig.Main)
			return err
		}
		var computeInvoker compute.Invoker
		mainExt := filepath.Ext(file)
		e.log.Info("Loading main program", "filename", file)
		switch mainExt {
		case ".wasm":
			computeInvoker, err = compute_wasmrs.Loader(e.ctx, map[string]interface{}{
				"filename": file,
			}, e.resolveAs)
		default:
			e.log.Error(err, "Unknown program type", "filename", file)
			return err
		}
		if err != nil {
			e.log.Error(err, "Error loading program", "filename", file)
			return err
		}
		e.m.Link(computeInvoker)
	}

	if err = processor.Initialize(busConfig); err != nil {
		e.log.Error(err, "Could not initialize processor")
		return err
	}

	// TODO: Figure out how to remove this
	for k, v := range processor.GetProviders() {
		e.allNamespaces[k] = v
	}
	for k, v := range processor.GetInterfaces() {
		e.allNamespaces[k] = v
	}

	// TODO: Lock down proivders
	e.m.Link(runtime.NewInvoker(e.log, processor.GetProviders(), e.codec))
	e.m.Link(runtime.NewInvoker(e.log, processor.GetInterfaces(), e.codec))

	return nil
}

func Start(ctx context.Context, info *Info) (*Engine, error) {
	// If there is a `.env` file, load its environment variables.
	if err := godotenv.Load(); err != nil {
		logger.Debug("could not load dotenv file", "error", err)
	}

	logger.SetLogLevel(info.LogLevel)
	log := logger.GetLogger()

	if info.DeveloperMode && info.Mode == ModeService {
		log.Info("Running in Developer Mode!")
	}
	// NanoBus flags
	// Pull from registry if PackageFile is set
	var busFile string
	if oci.IsImageReference(info.Target) {
		fmt.Printf("Pulling %s...\n", info.Target)
		var err error
		if busFile, err = oci.Pull(info.Target, "."); err != nil {
			fmt.Printf("Error pulling image: %s\n", err)
			return nil, err
		}

		if busFile == "" {
			fmt.Printf("Error in package: %s\n", "No BusFile returned")
			return nil, err
		}
	}

	if busFile == "" {
		busFile = info.Target
	}

	// Load the bus configuration
	busConfig, err := loadBusConfig(busFile, info.DeveloperMode, log)
	if err != nil {
		log.Error(err, "could not load configuration", "file", busFile)
		return nil, err
	}

	if busConfig.BaseURL == nil {
		// If we don't have a BaseUrl set, use CWD
		baseUrl, err := os.Getwd()
		if err != nil {
			// If we can't get the CWD, set to empty string and let the OS
			// sort it out
			baseUrl = ""
		}
		busConfig.BaseURL = &baseUrl
	}

	var resourcesConfig *runtime.ResourcesConfig
	// Load the resources configuration
	_, err = os.Stat(info.ResourcesFile)
	if err == nil {
		resourcesConfig, err = loadResourcesConfig(info.ResourcesFile, log)
		if err != nil {
			log.Error(err, "could not load configuration", "file", info.ResourcesFile)
			return nil, err
		}
	}

	// Transport registration
	transportRegistry := transport.Registry{}
	transportRegistry.Register(
		transport_dapr.DaprServerV1,
		transport_http.HttpServerV1,
		transport_httprpc.Load,
		transport_nats.Load,
		transport_time.SchedulerV1,
	)

	// Spec registration
	specRegistry := spec.Registry{}
	specRegistry.Register(
		spec_apex.Apex,
	)

	// Filter registration
	filterRegistry := filter.Registry{}
	filterRegistry.Register(
		jwt.JWTV1,
		paseto.PasetoV1,
		session.SessionV1,
		userinfo.UserInfoV1,
	)

	// Router registration
	routerRegistry := router.Registry{}
	routerRegistry.Register(
		router_oauth2.OAuth2V1,
		router_rest.RestV1,
		router_router.RouterV1,
		router_static.StaticV1,
	)

	middlewareRegistry := middleware.Registry{}
	middlewareRegistry.Register(
		middleware_cors.CorsV0,
	)

	// Compute registration
	computeRegistry := compute.Registry{}
	computeRegistry.Register(
		compute_wasmrs.WasmRS,
		// compute_mux.Mux,
		// compute_wapc.WaPC,
		// compute_rsocket.RSocket,
	)

	// Codec registration
	codecRegistry := codec.Registry{}
	codecRegistry.Register(
		codec_json.JSON,
		codec_msgpack.MsgPack,
		confluentavro.ConfluentAvro,
		cloudevents_avro.CloudEventsAvro,
		cloudevents_json.CloudEventsJSON,
		codec_text.Plain,
		codec_text.HTML,
		codec_bytes.Bytes,
	)

	resourceRegistry := resource.Registry{}
	resourceRegistry.Register(
		postgres.Connection,
		dapr.Client,
		blob.URLBlob,
		blob.AzureBlob,
		blob.FSBlob,
		blob.GCSBlob,
		blob.MemBlob,
		blob.S3Blob,
	)

	tracingRegistry := otel_tracing.Registry{}
	tracingRegistry.Register(
		tracing_jaeger.Jaeger,
		tracing_otlp.OTLP,
		tracing_stdout.Stdout,
	)

	// Action registration
	actionRegistry := actions.Registry{}
	actionRegistry.Register(core.All...)
	actionRegistry.Register(blob.All...)
	actionRegistry.Register(postgres.All...)
	actionRegistry.Register(dapr.All...)

	initializerRegistry := initialize.Registry{}
	initializerRegistry.Register(migrate_postgres.MigratePostgresV1)

	// Codecs
	jsoncodec := json_codec.New()
	msgpackcodec := msgpack_codec.New()
	bytescodec := bytes_codec.New()

	// Dependencies
	httpClient := getHTTPClient()
	env := getEnvironment()
	namespaces := make(spec.Namespaces)
	dependencies := map[string]interface{}{
		"system:logger": log,
		"system:application": &runtime.Application{
			ID:      busConfig.ID,
			Version: busConfig.Version,
		},
		"client:http":         httpClient,
		"codec:json":          jsoncodec,
		"codec:msgpack":       msgpackcodec,
		"codec:bytes":         bytescodec,
		"spec:namespaces":     namespaces,
		"os:env":              env,
		"registry:routers":    routerRegistry,
		"registry:middleware": middlewareRegistry,
		"developerMode":       info.DeveloperMode,
	}
	resolver := func(name string) (interface{}, bool) {
		dep, ok := dependencies[name]
		return dep, ok
	}
	resolveAs := resolve.ToResolveAs(resolver)

	var spanExporter sdk_trace.SpanExporter
	if info.Mode == ModeService && busConfig.Tracing != nil {
		log.Info("Initializing tracer", "type", busConfig.Tracing.Uses)
		loadable, ok := tracingRegistry[busConfig.Tracing.Uses]
		if !ok {
			log.Error(nil, "Could not find codec", "type", busConfig.Tracing.Uses)
			return nil, errors.New("cound not find codec")
		}
		var err error
		spanExporter, err = loadable(ctx, busConfig.Tracing.With, resolveAs)
		if err != nil {
			log.Error(err, "Error loading codec", "type", busConfig.Tracing.Uses)
			return nil, err
		}
	}

	var tp trace.TracerProvider

	if spanExporter == nil {
		tp = trace.NewNoopTracerProvider()
	} else {
		ntp := sdk_trace.NewTracerProvider(
			sdk_trace.WithBatcher(spanExporter),
			sdk_trace.WithResource(newOtelResource(busConfig.ID, busConfig.Version)),
		)
		defer func() {
			if err := ntp.Shutdown(ctx); err != nil {
				log.Error(err, "error shutting down trace provider")
			}
		}()
		tp = ntp
	}

	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.TraceContext{})
	tracer := otel.Tracer("NanoBus")
	dependencies["system:tracer"] = tracer

	if busConfig.Codecs == nil {
		busConfig.Codecs = map[string]runtime.Component{}
	}
	for name, loadable := range codecRegistry {
		if loadable.Auto {
			if _, exists := busConfig.Codecs[name]; !exists {
				busConfig.Codecs[name] = runtime.Component{
					Uses: name,
				}
			}
		}
	}

	codecs := make(codec.Codecs)
	codecsByContentType := make(codec.Codecs)
	for name, component := range busConfig.Codecs {
		log.Info("Initializing codec", "name", name, "type", component.Uses)
		loadable, ok := codecRegistry[component.Uses]
		if !ok {
			log.Error(nil, "Could not find codec", "type", component.Uses)
			return nil, errors.New("could not find codec")
		}
		c, err := loadable.Loader(component.With, resolveAs)
		if err != nil {
			log.Error(err, "Error loading codec", "type", component.Uses)
			return nil, err
		}
		codecs[name] = c
		codecsByContentType[c.ContentType()] = c
	}
	dependencies["codec:lookup"] = codecs
	dependencies["codec:byContentType"] = codecsByContentType

	authorizers := make(map[string]map[string]authorization.Rule)
	for iface, operations := range busConfig.Authorization {
		if len(operations) == 0 {
			continue
		}
		auths := make(map[string]authorization.Rule, len(operations))
		for operation, auth := range operations {
			if auth.Unauthenticated {
				auths[operation] = authorization.Unauthenticated
			} else {
				auths[operation] = authorization.NewBasic(auth.Has, auth.Checks)
			}
		}
		authorizers[iface] = auths
	}

	for name, spec := range busConfig.Initializers {
		log.Info("Initializer running", "name", name, "type", spec.Uses)
		loader, ok := initializerRegistry[spec.Uses]
		if !ok {
			log.Error(nil, "could not find initializer", "type", spec.Uses)
			return nil, err
		}
		if m, ok := spec.With.(map[string]interface{}); ok {
			m["name"] = name
		}
		nss, err := loader(ctx, spec.With, resolveAs)
		if err != nil {
			log.Error(err, "error loading initializer", "type", spec.Uses)
			return nil, err
		}
		if err := nss(ctx); err != nil {
			log.Error(err, "Could not execute initializer")
			return nil, err
		}
	}

	resources := resource.Resources{}
	if resourcesConfig != nil {
		for name, component := range resourcesConfig.Resources {
			log.Info("Initializing resource", "name", name, "type", component.Uses)

			loader, ok := resourceRegistry[component.Uses]
			if !ok {
				log.Error(nil, "Could not find resource", "type", component.Uses)
				return nil, err
			}
			c, err := loader(ctx, component.With, resolveAs)
			if err != nil {
				log.Error(err, "Error loading resource", "type", component.Uses)
				return nil, err
			}

			resources[name] = c
		}
	}
	dependencies["resource:lookup"] = resources

	dependencies["state:invoker"] = func(ctx context.Context, namespace, id, key string) ([]byte, error) {
		// TODO: Retrieve state
		return []byte{}, nil
	}

	m := mesh.New(tracer)
	dependencies["compute:mesh"] = m

	allNamespaces := make(runtime.Namespaces)
	dependencies["system:interfaces"] = allNamespaces

	// TODO(jsoverson): Remove. This is now unused. Keeping it for now as it
	// may have been a WIP
	// rt := Runtime{
	// 	log:        log,
	// 	config:     busConfig,
	// 	namespaces: namespaces,
	// 	processor:  allNamespaces,
	// 	resolver:   resolver,
	// 	resolveAs:  resolveAs,
	// 	env:        env,
	// }

	e := Engine{
		ctx:            ctx,
		log:            log,
		tracer:         tracer,
		actionRegistry: actionRegistry,
		resolver:       resolver,
		resolveAs:      resolveAs,
		namespaces:     namespaces,
		m:              m,
		allNamespaces:  allNamespaces,
		codec:          msgpackcodec,
	}

	if err := e.LoadConfig(busConfig); err != nil {
		return nil, err
	}

	for name, include := range busConfig.Imports {
		config, err := runtime.LoadIotaConfig(log, include.Ref, *busConfig.BaseURL)
		if err != nil {
			log.Error(err, "Could not read Iota config")
			return nil, err
		}
		includedConfig := &runtime.BusConfig{
			ID:         name,
			Resources:  config.Resources,
			Version:    config.Version,
			Main:       config.Main,
			Spec:       config.Spec,
			Interfaces: config.Interfaces,
			Providers:  config.Providers,
			BaseURL:    config.BaseURL,
		}

		if err := e.LoadConfig(includedConfig); err != nil {
			return nil, err
		}
	}

	interfaces := namespaces.ToInterfaces()

	// Check for unsatified imports
	ops := m.Unsatisfied()
	if len(ops) > 0 {
		log.Error(nil, "Halting due to unsatified imports", "count", len(ops))
		for _, op := range ops {
			log.Error(nil, "Missing import", "interface", op.Namespace, "operation", op.Operation)
		}
		return nil, errors.New("halting due to unsatified imports")
	}

	filters := []filter.Filter{}
	for _, f := range busConfig.Filters {
		filterLoader, ok := filterRegistry[f.Uses]
		if !ok {
			log.Error(nil, "could not find filter", "type", f.Uses)
			return nil, errors.New("could not find filter")
		}

		filter, err := filterLoader(ctx, f.With, resolveAs)
		if err != nil {
			log.Error(err, "could not load filter", "type", f.Uses)
			return nil, err
		}

		filters = append(filters, filter)
	}
	dependencies["filter:lookup"] = filters

	translateError := func(err error) *errorz.Error {
		if errz, ok := err.(*errorz.Error); ok {
			return errz
		}
		var te errorz.TemplateError

		if terrz, ok := err.(*errorz.TemplateError); ok && terrz != nil {
			te = *terrz
		} else {
			te = errorz.ParseTemplateError(err.Error())
		}

		tmpl, ok := busConfig.Errors[te.Template]
		if !ok {
			// Default error if template matches a code name.
			if code, ok := errorz.CodeLookup[te.Template]; ok {
				return errorz.New(code)
			}

			return errorz.New(errorz.Internal, err.Error())
		}

		message := err.Error()
		if tmpl.Message != nil {
			message, _ = tmpl.Message.Eval(te.Metadata)
		}

		e := errorz.New(errorz.ErrCode(tmpl.Code), message)
		e.Type = te.Template
		if tmpl.Type != "" {
			e.Type = tmpl.Type
		}
		if tmpl.Status != 0 {
			e.Status = int(tmpl.Status)
		}
		if tmpl.Title != nil {
			title, _ := tmpl.Title.Eval(te.Metadata)
			e.Title = title
		}
		if tmpl.Help != nil {
			instance, _ := tmpl.Help.Eval(te.Metadata)
			e.Help = instance
		}
		e.Metadata = te.Metadata

		return e
	}
	dependencies["errors:resolver"] = errorz.Resolver(translateError)

	// healthHandler := func(w http.ResponseWriter, r *http.Request) {
	// 	//fmt.Println("HEALTH HANDLER CALLED")
	// 	defer r.Body.Close()

	// 	w.Header().Set("Content-Type", "text/plain")
	// 	w.Write([]byte("OK"))
	// }

	transportInvoker := func(ctx context.Context, h handler.Handler, id string, input interface{}, authorization transport.Authorization) (interface{}, error) {
		if err := coalesceInput(interfaces, h, input); err != nil {
			return nil, err
		}

		claimsMap := claims.FromContext(ctx)

		if authorization != transport.BypassAuthorization {
			// Perform authorization first.
			// Deny by default of no rule is found for the operation.
			opers, ok := authorizers[h.Interface]
			if !ok {
				return nil, errorz.Return("permission_denied", errorz.Metadata{})
			}
			oper, ok := opers[h.Operation]
			if !ok {
				return nil, errorz.Return("permission_denied", errorz.Metadata{})
			}
			if err := oper.Check(claimsMap); err != nil {
				return nil, err // wrapped by Check
			}
		}

		data := actions.Data{
			"claims": claimsMap,
			"input":  input,
			"$":      input,
			"pipe":   input,
		}

		if jsonBytes, err := json.MarshalIndent(input, "", "  "); err == nil {
			logInbound(h, string(jsonBytes))
		}

		data["env"] = env

		ctx = handler.ToContext(ctx, h)

		// TODO: Use merged map of interfaces here
		response, ok, err := allNamespaces.Invoke(ctx, h, data)
		if err != nil {
			return nil, translateError(err)
		}

		// No pipeline exits for the operation so invoke directly.
		if !ok {
			payloadData, err := msgpack.Marshal(input)
			if err != nil {
				return nil, translateError(err)
			}

			metadata := make([]byte, 8)
			p := payload.New(payloadData, metadata)

			future := m.RequestResponse(ctx, h, p)
			if future == nil {
				return nil, errorz.New(errorz.Unimplemented, fmt.Sprintf("%s is not implemented", h.String()))
			}
			result, err := future.Block()
			if err != nil {
				return nil, translateError(err)
			}

			if len(result.Data()) > 0 {
				var resultDecoded interface{}
				if err := msgpack.Unmarshal(result.Data(), &resultDecoded); err != nil {
					return nil, translateError(err)
				}
				response = resultDecoded
			}
		}

		return response, err
	}
	e.transportInvoker = transportInvoker
	dependencies["transport:invoker"] = transport.Invoker(transportInvoker)

	if info.Mode == ModeService {
		if len(busConfig.Transports) == 0 {
			log.Info("Warning: no transports configured")
		}

		var g run.Group
		if len(info.Process) > 0 {
			log.Info("Executing process", "cmd", strings.Join(info.Process, " "))
			command := info.Process[0]
			args := info.Process[1:]
			cmd := exec.CommandContext(ctx, command, args...)
			g.Add(func() error {
				appEnv := []string{}
				env := []string{}
				env = append(env, os.Environ()...)
				env = append(env, appEnv...)
				cmd.Env = env
				cmd.Stdin = os.Stdin
				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr
				return cmd.Run()
			}, func(error) {
				// TODO: send term sig instead
				if cmd.Process != nil {
					if err := cmd.Process.Kill(); err != nil {
						logger.Error("Error killing process", "error", err)
					}
				}
			})
		}
		{
			g.Add(func() error {
				return m.WaitUntilShutdown()
			}, func(error) {
				m.Close()
			})
		}

		for name, comp := range busConfig.Transports {
			name := name // Make copy
			loader, ok := transportRegistry[comp.Uses]
			if !ok {
				log.Error(nil, "unknown transport", "type", comp.Uses)
				return nil, err
			}
			log.Info("Initializing transport", "name", name)
			t, err := loader(ctx, comp.With, resolveAs)
			if err != nil {
				log.Error(err, "could not load transport", "type", comp.Uses)
				return nil, err
			}

			g.Add(func() error {
				return t.Listen()
			}, func(error) {
				t.Close()
			})
		}

		{
			g.Add(run.SignalHandler(ctx, syscall.SIGINT, syscall.SIGTERM))
		}

		err = g.Run()
		log.Info("Shutting down")
		if err != nil {
			if _, isSignal := err.(run.SignalError); !isSignal {
				log.Error(err, "unexpected error")
			}
		}
	}

	return &e, nil
}

func (e *Engine) InvokeUnsafe(handler handler.Handler, input any) (any, error) {
	return e.transportInvoker(e.ctx, handler, "", input, transport.BypassAuthorization)
}

func (e *Engine) Invoke(handler handler.Handler, input any) (any, error) {
	return e.transportInvoker(e.ctx, handler, "", input, transport.PerformAuthorization)
}

func loadResourcesConfig(filename string, log logr.Logger) (*runtime.ResourcesConfig, error) {
	// TODO: Load from file or URI
	f, err := os.OpenFile(filename, os.O_RDONLY, 0644)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	c, err := runtime.LoadResourcesYAML(f)
	if err != nil {
		return nil, err
	}

	return c, nil
}

func loadBusConfig(filename string, developerMode bool, log logr.Logger) (*runtime.BusConfig, error) {
	var in io.Reader
	if strings.HasSuffix(filename, ".ts") {
		if !developerMode {
			return nil, errors.New("loading configuration directly from TypeScript is only allowed in developer mode")
		}
		out, err := exec.Command("deno", "run", "--allow-run", "--unstable", filename).Output()
		if err != nil {
			return nil, fmt.Errorf("error running bus program: %w", err)
		}
		in = bytes.NewReader(out)
	} else {
		// TODO: Load from file or URI
		f, err := os.OpenFile(filename, os.O_RDONLY, 0644)
		if err != nil {
			return nil, err
		}
		in = f
		defer f.Close()
	}

	absPath, err := filepath.Abs(filename)
	if err != nil {
		return nil, err
	}
	baseDir := filepath.Dir(absPath)

	c, err := runtime.LoadBusYAML(baseDir, in)
	if err != nil {
		return nil, err
	}

	// for _, imp := range c.Import {
	// 	fileDir := filepath.Dir(imp)
	// 	path := filepath.Join(baseDir, imp)
	// 	rel := runtime.FilePath(path)
	// 	log.Info("Importing config", "config", rel.Relative())
	// 	dir := filepath.Dir(path)
	// 	runtime.SetConfigBaseDir(dir)
	// 	imported, err := loadConfiguration(path, log)
	// 	if err != nil {
	// 		return nil, err
	// 	}
	// 	runtime.Combine(c, fileDir, log, imported)
	// 	runtime.SetConfigBaseDir(baseDir)
	// }

	return c, nil
}

func getHTTPClient() *http.Client {
	t := http.DefaultTransport.(*http.Transport).Clone()
	t.MaxIdleConns = 100
	t.MaxConnsPerHost = 100
	t.MaxIdleConnsPerHost = 100

	return &http.Client{
		Timeout:   10 * time.Second,
		Transport: t,
	}
}

func getEnvironment() runtime.Environment {
	return environmentToMap(os.Environ(), func(item string) (key, val string) {
		splits := strings.SplitN(item, "=", 1)
		key = splits[0]
		if len(splits) > 1 {
			val = splits[1]
		}

		return
	})
}

func environmentToMap(environment []string, getkeyval func(item string) (key, val string)) map[string]string {
	items := make(map[string]string)
	for _, item := range environment {
		key, val := getkeyval(item)
		items[key] = val
	}

	return items
}

func coalesceInput(interfaces spec.Interfaces, h handler.Handler, input interface{}) error {
	if oper, ok := interfaces.Operation(h); ok {
		if oper.Parameters != nil {
			inputMap, ok := coalesce.ToMapSI(input, true)
			if !ok {
				return fmt.Errorf("%w: input is not a map", transport.ErrBadInput)
			}
			if err := oper.Parameters.Coalesce(inputMap, true); err != nil {
				var errz *errorz.Error
				if errors.As(err, &errz) {
					return errz
				}
				return fmt.Errorf("%w: %v", transport.ErrBadInput, err)
			}
		}
	} else {
		coalesce.Integers(input)
	}
	return nil
}

// TODO(jsoverson): Eventually delete or use. Saving to avoid
// reimplementing if it's used in the near future
// func coalesceOutput(namespaces spec.Namespaces, namespace, service, function string, output interface{}) (interface{}, error) {
// 	var err error
// 	if oper, ok := namespaces.Operation(namespace, service, function); ok {
// 		if oper.Returns != nil && output != nil {
// 			output, _, err = oper.Returns.Coalesce(output, false)
// 			if err != nil {
// 				return nil, err
// 			}
// 		} else {
// 			coalesce.Integers(output)
// 		}
// 	} else {
// 		coalesce.Integers(output)
// 	}
// 	return output, err
// }

func logInbound(h handler.Handler, data string) {
	logger.Debug("==> " + h.String() + " " + data)
}

// TODO(jsoverson): Eventually delete or use. Saving to avoid
// reimplementing if it's used in the near future
// func logOutbound(target string, data string) {
// 	logger.Debug("<== " + target + " " + data)
// }

// newOtelResource returns a resource describing this application.
func newOtelResource(applicationID, version string) *otel_resource.Resource {
	r, _ := otel_resource.Merge(
		otel_resource.Default(),
		otel_resource.NewWithAttributes(
			semconv.SchemaURL,
			semconv.ServiceNameKey.String(applicationID),
			semconv.ServiceVersionKey.String(version),
		),
	)
	return r
}
