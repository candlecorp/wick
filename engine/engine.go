package engine

import (
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
	"github.com/go-logr/zapr"
	"github.com/joho/godotenv"
	"github.com/mattn/go-colorable"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"

	"github.com/oklog/run"
	"github.com/vmihailenco/msgpack/v5"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/propagation"
	otel_resource "go.opentelemetry.io/otel/sdk/resource"
	sdk_trace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.12.0"
	"go.opentelemetry.io/otel/trace"

	// COMPONENT MODEL / PLUGGABLE COMPONENTS
	proto "github.com/dapr/dapr/pkg/proto/components/v1"

	// NANOBUS CORE
	"github.com/nanobus/nanobus/coalesce"
	"github.com/nanobus/nanobus/errorz"
	"github.com/nanobus/nanobus/function"
	"github.com/nanobus/nanobus/mesh"
	"github.com/nanobus/nanobus/migrate"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/resource"
	"github.com/nanobus/nanobus/runtime"
	"github.com/nanobus/nanobus/security/claims"

	// CHANNELS
	json_codec "github.com/nanobus/nanobus/channel/codecs/json"
	msgpack_codec "github.com/nanobus/nanobus/channel/codecs/msgpack"

	// SPECIFICATION LANGUAGES
	"github.com/nanobus/nanobus/spec"
	spec_apex "github.com/nanobus/nanobus/spec/apex"

	// COMPONENTS
	"github.com/nanobus/iota/go/wasmrs/payload"
	"github.com/nanobus/nanobus/compute"
	compute_wasmrs "github.com/nanobus/nanobus/compute/wasmrs"

	// ACTIONS
	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/actions/core"
	"github.com/nanobus/nanobus/actions/dapr"
	"github.com/nanobus/nanobus/actions/gorm"
	"github.com/nanobus/nanobus/actions/postgres"

	// CODECS
	"github.com/nanobus/nanobus/codec"
	cloudevents_avro "github.com/nanobus/nanobus/codec/cloudevents/avro"
	cloudevents_json "github.com/nanobus/nanobus/codec/cloudevents/json"
	"github.com/nanobus/nanobus/codec/confluentavro"
	codec_json "github.com/nanobus/nanobus/codec/json"
	codec_msgpack "github.com/nanobus/nanobus/codec/msgpack"
	codec_text "github.com/nanobus/nanobus/codec/text"

	// DB MIGRATION
	migrate_postgres "github.com/nanobus/nanobus/migrate/postgres"

	// TELEMETRY / TRACING
	otel_tracing "github.com/nanobus/nanobus/telemetry/tracing"
	tracing_jaeger "github.com/nanobus/nanobus/telemetry/tracing/jaeger"
	tracing_otlp "github.com/nanobus/nanobus/telemetry/tracing/otlp"
	tracing_stdout "github.com/nanobus/nanobus/telemetry/tracing/stdout"

	// TRANSPORTS
	"github.com/nanobus/nanobus/transport"
	"github.com/nanobus/nanobus/transport/httprpc"
	"github.com/nanobus/nanobus/transport/nats"
	"github.com/nanobus/nanobus/transport/rest"

	// TRANSPORT - FILTERS
	"github.com/nanobus/nanobus/transport/filter"
	"github.com/nanobus/nanobus/transport/filter/jwt"
	"github.com/nanobus/nanobus/transport/filter/session"
	"github.com/nanobus/nanobus/transport/filter/userinfo"

	// TRANSPORT - MIDDLEWARE
	"github.com/nanobus/nanobus/transport/middleware"
	middleware_cors "github.com/nanobus/nanobus/transport/middleware/cors"

	// TRANSPORT - ROUTES
	"github.com/nanobus/nanobus/transport/routes"
	"github.com/nanobus/nanobus/transport/routes/oauth2"
)

type Runtime struct {
	log        logr.Logger
	config     *runtime.Configuration
	namespaces spec.Namespaces
	processor  *runtime.Processor
	resolver   resolve.DependencyResolver
	resolveAs  resolve.ResolveAs
	env        runtime.Environment
}

type Mode int

const (
	ModeService Mode = iota
	ModeInvoke
)

type Info struct {
	Mode    Mode
	BusFile string

	// Service mode
	Process []string

	// CLI mode
	Namespace string
	Service   string
	EntityID  string
	Operation string
	Input     any
	Output    any
}

func Start(info *Info) error {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// If there is a `.env` file, load its environment variables.
	godotenv.Load()

	logLevel := zapcore.DebugLevel
	if info.Mode == ModeInvoke {
		logLevel = zap.ErrorLevel
	}

	// Initialize logger
	zapConfig := zap.NewDevelopmentEncoderConfig()
	zapConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	zapLog := zap.New(zapcore.NewCore(
		zapcore.NewConsoleEncoder(zapConfig),
		zapcore.AddSync(colorable.NewColorableStdout()),
		logLevel,
	))
	//zapLog, err := zapConfig.Build()
	//zapLog, err := zap.NewProduction()
	// if err != nil {
	// 	panic(err)
	// }
	// zapLog := zap.NewExample()
	log := zapr.NewLogger(zapLog)

	// NanoBus flags

	// Load the configuration
	config, err := loadConfiguration(info.BusFile, log)
	if err != nil {
		log.Error(err, "could not load configuration", "file", info.BusFile)
		return err
	}

	// Transport registration
	transportRegistry := transport.Registry{}
	transportRegistry.Register(
		httprpc.Load,
		rest.Load,
		nats.Load,
	)

	// Spec registration
	specRegistry := spec.Registry{}
	specRegistry.Register(
		spec_apex.Apex,
	)

	// Routes registration
	routesRegistry := routes.Registry{}
	routesRegistry.Register(
		oauth2.Oauth2,
	)

	// Filter registration
	filterRegistry := filter.Registry{}
	filterRegistry.Register(
		jwt.JWT,
		session.Session,
		userinfo.UserInfo,
	)

	middlewareRegistry := middleware.Registry{}
	middlewareRegistry.Register(
		middleware_cors.Cors,
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
	)

	resourceRegistry := resource.Registry{}
	resourceRegistry.Register(
		postgres.Connection,
		gorm.Connection,
		dapr.PubSub,
		dapr.StateStore,
		dapr.OutputBinding,
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
	actionRegistry.Register(postgres.All...)
	actionRegistry.Register(gorm.All...)
	actionRegistry.Register(dapr.All...)

	migrateRegistry := migrate.Registry{}
	migrateRegistry.Register(migrate_postgres.NamedLoader)

	// Codecs
	jsoncodec := json_codec.New()
	msgpackcodec := msgpack_codec.New()

	// Dependencies
	// var invoker *channel.Invoker
	// var busInvoker compute.BusInvoker
	httpClient := getHTTPClient()
	env := getEnvironment()
	namespaces := make(spec.Namespaces)
	dependencies := map[string]interface{}{
		"system:logger":   log,
		"client:http":     httpClient,
		"codec:json":      jsoncodec,
		"codec:msgpack":   msgpackcodec,
		"spec:namespaces": namespaces,
		"os:env":          env,
		"registry:routes": routesRegistry,
	}
	resolver := func(name string) (interface{}, bool) {
		dep, ok := dependencies[name]
		return dep, ok
	}
	resolveAs := resolve.ToResolveAs(resolver)

	var spanExporter sdk_trace.SpanExporter
	if info.Mode == ModeService && config.Tracing != nil {
		loadable, ok := tracingRegistry[config.Tracing.Uses]
		if !ok {
			log.Error(nil, "Could not find codec", "type", config.Tracing.Uses)
			return errors.New("cound not find codec")
		}
		var err error
		spanExporter, err = loadable(ctx, config.Tracing.With, resolveAs)
		if err != nil {
			log.Error(err, "Error loading codec", "type", config.Tracing.Uses)
			return err
		}
	}

	var tp trace.TracerProvider

	if spanExporter == nil {
		tp = trace.NewNoopTracerProvider()
	} else {
		ntp := sdk_trace.NewTracerProvider(
			sdk_trace.WithBatcher(spanExporter),
			sdk_trace.WithResource(newOtelResource(config.Application)),
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

	// if len(config.Specs) == 0 {
	// 	config.Specs = append(config.Specs, runtime.Component{
	// 		Uses: "apex",
	// 		With: map[string]interface{}{
	// 			"filename": "spec.apexlang",
	// 		},
	// 	})
	// }
	for _, spec := range config.Specs {
		loader, ok := specRegistry[spec.Uses]
		if !ok {
			log.Error(nil, "Could not find spec", "type", spec.Uses)
			return errors.New("could not find spec")
		}
		nss, err := loader(ctx, spec.With, resolveAs)
		if err != nil {
			log.Error(err, "Error loading spec", "type", spec.Uses)
			return err
		}
		for _, ns := range nss {
			namespaces[ns.Name] = ns
		}
	}

	if config.Codecs == nil {
		config.Codecs = map[string]runtime.Component{}
	}
	for name, loadable := range codecRegistry {
		if loadable.Auto {
			if _, exists := config.Codecs[name]; !exists {
				config.Codecs[name] = runtime.Component{
					Uses: name,
				}
			}
		}
	}

	codecs := make(codec.Codecs)
	codecsByContentType := make(codec.Codecs)
	for name, component := range config.Codecs {
		loadable, ok := codecRegistry[component.Uses]
		if !ok {
			log.Error(nil, "Could not find codec", "type", component.Uses)
			return errors.New("could not find codec")
		}
		c, err := loadable.Loader(component.With, resolveAs)
		if err != nil {
			log.Error(err, "Error loading codec", "type", component.Uses)
			return err
		}
		codecs[name] = c
		codecsByContentType[c.ContentType()] = c
	}
	dependencies["codec:lookup"] = codecs
	dependencies["codec:byContentType"] = codecsByContentType

	for name, spec := range config.Migrate {
		log.Info("Migrating database", "name", name)
		loader, ok := migrateRegistry[spec.Uses]
		if !ok {
			log.Error(nil, "could not find migrater", "type", spec.Uses)
			return err
		}
		if m, ok := spec.With.(map[string]interface{}); ok {
			m["name"] = name
		}
		nss, err := loader(ctx, spec.With, resolveAs)
		if err != nil {
			log.Error(err, "error loading migrater", "type", spec.Uses)
			return err
		}
		if err := nss(ctx); err != nil {
			log.Error(err, "Could not migrate database")
			return err
		}
	}

	resources := resource.Resources{}
	for name, component := range config.Resources {
		log.Info("Initializing resource", "name", name)

		loader, ok := resourceRegistry[component.Uses]
		if !ok {
			log.Error(nil, "Could not find resource", "type", component.Uses)
			return err
		}
		c, err := loader(ctx, component.With, resolveAs)
		if err != nil {
			log.Error(err, "Error loading resource", "type", component.Uses)
			return err
		}

		resources[name] = c
	}
	dependencies["resource:lookup"] = resources

	// Create processor
	processor, err := runtime.NewProcessor(ctx, log, tracer, config, actionRegistry, resolver)
	if err != nil {
		log.Error(err, "Could not create NanoBus runtime")
		return err
	}
	dependencies["system:processor"] = processor

	rt := Runtime{
		log:        log,
		config:     config,
		namespaces: namespaces,
		processor:  processor,
		resolver:   resolver,
		resolveAs:  resolveAs,
		env:        env,
	}
	// busInvoker = rt.BusInvoker
	// dependencies["bus:invoker"] = busInvoker
	dependencies["state:invoker"] = func(ctx context.Context, namespace, id, key string) ([]byte, error) {
		// TODO: Retrieve state
		return []byte{}, nil
	}

	m := mesh.New(tracer)

	for _, comp := range config.Compute {
		log.Info("Initializing compute", "type", comp.Uses, "with", comp.With)
		computeLoader, ok := computeRegistry[comp.Uses]
		if !ok {
			log.Error(err, "could not find compute", "type", comp.Uses)
			return err
		}
		invoker, err := computeLoader(ctx, comp.With, resolveAs)
		if err != nil {
			log.Error(err, "could not load compute", "type", comp.Uses)
			return err
		}
		m.Link(invoker)
	}
	dependencies["compute:mesh"] = m

	if err = processor.Initialize(); err != nil {
		log.Error(err, "Could not initialize processor")
		return err
	}

	m.Link(runtime.NewInvoker(log, processor.GetProviders(), msgpackcodec))

	// Check for unsatified imports
	ops := m.Unsatisfied()
	if len(ops) > 0 {
		log.Error(nil, "Halting due to unsatified imports", "count", len(ops))
		for _, op := range ops {
			log.Error(nil, "Missing import", "namespace", op.Namespace, "operation", op.Operation)
		}
		return errors.New("halting due to unsatified imports")
	}

	// log.Info(strings.TrimSpace(m.DebugInfo()))

	for _, subscription := range config.Subscriptions {
		pubsub, err := resource.Get[proto.PubSubClient](resources, subscription.Resource)
		if err != nil {
			log.Error(err, "Could not load resource", "name", subscription.Resource)
			return err
		}

		c, ok := codecs[subscription.Codec]
		if !ok {
			log.Error(nil, "Could not find codec", "name", subscription.Resource)
			return errors.New("could not find codec")
		}

		pull, err := pubsub.PullMessages(ctx)
		if err != nil {
			log.Error(nil, "Could not pull messages", "name", subscription.Resource)
			return errors.New("could not pull messages")
		}

		go func(pull proto.PubSub_PullMessagesClient, c codec.Codec, sub runtime.Subscription) {
			if err := pull.Send(&proto.PullMessagesRequest{
				Topic: &proto.Topic{
					Name:     sub.Topic,
					Metadata: sub.Metadata,
				},
			}); err != nil {
				log.Error(err, "Error subscribing")
				return
			}

			log.Info("Subscribed to pubsub", "resource", sub.Resource, "topic", sub.Topic)

			for {
				recv, err := pull.Recv()
				if err == io.EOF || err == context.Canceled {
					return
				}
				if err != nil {
					log.Error(err, "Error receiving messages")
					return
				}

				input, messageType, err := c.Decode(recv.Data, sub.CodecArgs...)
				if err != nil {
					log.Error(err, "could not decode message")
					pull.Send(&proto.PullMessagesRequest{
						AckMessageId: recv.Id,
						AckError: &proto.AckMessageError{
							Message: err.Error(),
						},
					})
					continue
				}
				// Extract distributed tracing context
				// per the the W3C TraceContext standard.
				if m, ok := input.(map[string]interface{}); ok {
					ctx = otel.GetTextMapPropagator().Extract(ctx, otel_tracing.MapCarrier(m))
				}

				data := actions.Data{
					"input": input,
				}

				pipelineName := sub.Function
				if pipelineName == "" {
					pipelineName = messageType
				}

				traceName := "events::type=" + pipelineName
				if jsonBytes, err := json.MarshalIndent(input, "", "  "); err == nil {
					logInbound(log, traceName, string(jsonBytes))
				}

				var span trace.Span
				ctx, span = tracer.Start(ctx, traceName, trace.WithAttributes(
					semconv.MessagingOperationProcess,
				))
				_, err = processor.Event(ctx, pipelineName, data)
				if err != nil {
					log.Error(err, "could not process message")
					pull.Send(&proto.PullMessagesRequest{
						AckMessageId: recv.Id,
						AckError: &proto.AckMessageError{
							Message: err.Error(),
						},
					})
					span.RecordError(err)
					span.End()
					continue
				}

				if err := pull.Send(&proto.PullMessagesRequest{
					AckMessageId: recv.Id,
				}); err != nil {
					log.Error(err, "could not ack message", "messageId", recv.Id)
					span.RecordError(err)
				}
				span.End()
			}
		}(pull, c, subscription)
	}

	// Big 'ol TODO
	// invoker = computeInstance.Invoker
	// dependencies["client:invoker"] = invoker

	filters := []filter.Filter{}
	if configFilters, ok := config.Filters["http"]; ok {
		for _, f := range configFilters {
			filterLoader, ok := filterRegistry[f.Uses]
			if !ok {
				log.Error(nil, "could not find filter", "type", f.Uses)
				return errors.New("could not find filter")
			}

			filter, err := filterLoader(ctx, f.With, resolveAs)
			if err != nil {
				log.Error(err, "could not load filter", "type", f.Uses)
				return err
			}

			filters = append(filters, filter)
		}
	}
	dependencies["filter:lookup"] = filters

	middlewares := []middleware.Middleware{}
	if configMiddlewares, ok := config.Middleware["http"]; ok {
		for _, f := range configMiddlewares {
			middlewareLoader, ok := middlewareRegistry[f.Uses]
			if !ok {
				log.Error(nil, "could not find middleware", "type", f.Uses)
				return errors.New("could not find middleware")
			}

			middleware, err := middlewareLoader(ctx, f.With, resolveAs)
			if err != nil {
				log.Error(err, "could not load middleware", "type", f.Uses)
				return err
			}

			middlewares = append(middlewares, middleware)
		}
	}
	dependencies["middleware:lookup"] = middlewares

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

		tmpl, ok := config.Errors[te.Template]
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

		e := errorz.New(tmpl.Code, message)
		e.Type = te.Template
		if tmpl.Type != "" {
			e.Type = tmpl.Type
		}
		if tmpl.Status != 0 {
			e.Status = tmpl.Status
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

	transportInvoker := func(ctx context.Context, namespace, service, id, fn string, input interface{}) (interface{}, error) {
		if err := coalesceInput(namespaces, namespace, service, fn, input); err != nil {
			return nil, err
		}

		claimsMap := claims.FromContext(ctx)

		data := actions.Data{
			"claims": claimsMap,
			"input":  input,
		}

		ns := namespace
		if service != "" {
			ns += "." + service
		}

		if jsonBytes, err := json.MarshalIndent(input, "", "  "); err == nil {
			logInbound(rt.log, ns+"/"+fn, string(jsonBytes))
		}

		data["env"] = env

		ctx = function.ToContext(ctx, function.Function{
			Namespace: ns,
			Operation: fn,
		})

		response, ok, err := rt.processor.Service(ctx, namespace, service, fn, data)
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

			future := m.RequestResponse(ctx, ns, fn, p)
			if future == nil {
				return nil, errorz.New(errorz.Unimplemented, fmt.Sprintf("%s::%s is not implemented", ns, fn))
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
	dependencies["transport:invoker"] = transport.Invoker(transportInvoker)

	switch info.Mode {
	case ModeService:
		if len(config.Transports) == 0 {
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
					cmd.Process.Kill()
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

		for name, comp := range config.Transports {
			name := name // Make copy
			loader, ok := transportRegistry[comp.Uses]
			if !ok {
				log.Error(nil, "unknown transport", "type", comp.Uses)
				return err
			}
			log.Info("Initializing transport", "name", name)
			t, err := loader(ctx, comp.With, resolveAs)
			if err != nil {
				log.Error(err, "could not load transport", "type", comp.Uses)
				return err
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

	case ModeInvoke:
		result, err := transportInvoker(ctx,
			info.Namespace,
			info.Service,
			info.EntityID,
			info.Operation,
			info.Input)
		if err != nil {
			log.Error(err, "invoke failed")
			return err
		}

		info.Output = result
	}

	return nil
}

func loadConfiguration(filename string, log logr.Logger) (*runtime.Configuration, error) {
	// TODO: Load from file or URI
	f, err := os.OpenFile(filename, os.O_RDONLY, 0644)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	absPath, err := filepath.Abs(filename)
	if err != nil {
		return nil, err
	}
	baseDir := filepath.Dir(absPath)

	c, err := runtime.LoadYAML(f)
	if err != nil {
		return nil, err
	}

	for _, imp := range c.Import {
		fileDir := filepath.Dir(imp)
		path := filepath.Join(baseDir, imp)
		rel := runtime.FilePath(path)
		log.Info("Importing config", "config", rel.Relative())
		dir := filepath.Dir(path)
		runtime.SetConfigBaseDir(dir)
		imported, err := loadConfiguration(path, log)
		if err != nil {
			return nil, err
		}
		runtime.Combine(c, fileDir, log, imported)
		runtime.SetConfigBaseDir(baseDir)
	}

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

func coalesceInput(namespaces spec.Namespaces, namespace, service, function string, input interface{}) error {
	if oper, ok := namespaces.Operation(namespace, service, function); ok {
		if oper.Parameters != nil {
			inputMap, ok := coalesce.ToMapSI(input, true)
			if !ok {
				return fmt.Errorf("%w: input is not a map", transport.ErrBadInput)
			}
			input = inputMap
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

func coalesceOutput(namespaces spec.Namespaces, namespace, service, function string, output interface{}) (interface{}, error) {
	var err error
	if oper, ok := namespaces.Operation(namespace, service, function); ok {
		if oper.Returns != nil && output != nil {
			output, _, err = oper.Returns.Coalesce(output, false)
			if err != nil {
				return nil, err
			}
		} else {
			coalesce.Integers(output)
		}
	} else {
		coalesce.Integers(output)
	}
	return output, err
}

func logInbound(log logr.Logger, target string, data string) {
	l := log //.V(10)
	if l.Enabled() {
		l.Info("==> " + target + " " + data)
	}
}

func logOutbound(log logr.Logger, target string, data string) {
	l := log //.V(10)
	if l.Enabled() {
		l.Info("<== " + target + " " + data)
	}
} // )

// newOtelResource returns a resource describing this application.
func newOtelResource(app *runtime.Application) *otel_resource.Resource {
	serviceKey := "nanobus"
	version := ""
	environment := "non-prod"

	if app != nil {
		if app.ID != "" {
			serviceKey = app.ID
		}
		if app.Version != "" {
			version = app.Version
		}
		if app.Environment != "" {
			environment = app.Environment
		}
	}

	r, _ := otel_resource.Merge(
		otel_resource.Default(),
		otel_resource.NewWithAttributes(
			semconv.SchemaURL,
			semconv.ServiceNameKey.String(serviceKey),
			semconv.ServiceVersionKey.String(version),
			attribute.String("environment", environment),
		),
	)
	return r
}
