package nats

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strconv"

	"github.com/go-logr/logr"
	"github.com/nats-io/nats.go"
	"go.uber.org/multierr"

	"github.com/nanobus/nanobus/channel"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/errorz"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/spec"
	"github.com/nanobus/nanobus/transport"
	"github.com/nanobus/nanobus/transport/filter"
)

type NATS struct {
	log           logr.Logger
	ctx           context.Context
	cancel        context.CancelFunc
	nc            *nats.Conn
	namespaces    spec.Namespaces
	invoker       transport.Invoker
	errorResolver errorz.Resolver
	codecs        map[string]channel.Codec
	filters       []filter.Filter
	subs          []*nats.Subscription
}

type optionsHolder struct {
	codecs  []channel.Codec
	filters []filter.Filter
}

var (
	ErrUnregisteredContentType = errors.New("unregistered content type")
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

type Configuration struct {
	Address string `mapstructure:"address" validate:"required"`
}

func Load() (string, transport.Loader) {
	return "nats", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (transport.Transport, error) {
	var jsoncodec channel.Codec
	var msgpackcodec channel.Codec
	var transportInvoker transport.Invoker
	var namespaces spec.Namespaces
	var errorResolver errorz.Resolver
	var filters []filter.Filter
	var log logr.Logger
	if err := resolve.Resolve(resolver,
		"codec:json", &jsoncodec,
		"codec:msgpack", &msgpackcodec,
		"transport:invoker", &transportInvoker,
		"spec:namespaces", &namespaces,
		"errors:resolver", &errorResolver,
		"filter:lookup", &filters,
		"system:logger", &log); err != nil {
		return nil, err
	}

	var c Configuration
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return New(log, c.Address, namespaces, transportInvoker, errorResolver,
		WithFilters(filters...),
		WithCodecs(jsoncodec, msgpackcodec))
}

func New(log logr.Logger, address string, namespaces spec.Namespaces, invoker transport.Invoker, errorResolver errorz.Resolver, options ...Option) (transport.Transport, error) {
	var opts optionsHolder

	for _, opt := range options {
		opt(&opts)
	}

	codecMap := make(map[string]channel.Codec, len(opts.codecs))
	for _, c := range opts.codecs {
		codecMap[c.ContentType()] = c
	}

	ctx, cancel := context.WithCancel(context.Background())
	nc, err := nats.Connect(address)
	if err != nil {
		cancel()
		return nil, err
	}

	log.Info("Connected to NATS", "address", address)

	return &NATS{
		log:           log,
		ctx:           ctx,
		cancel:        cancel,
		nc:            nc,
		namespaces:    namespaces,
		invoker:       invoker,
		errorResolver: errorResolver,
		codecs:        codecMap,
		filters:       opts.filters,
	}, nil
}

func (t *NATS) Listen() error {
	subs := make([]*nats.Subscription, 0, len(t.namespaces))
	for ns := range t.namespaces {
		t.log.Info("Subscribing", "namespace", ns)
		sub, err := t.nc.Subscribe(ns+".>", t.handler)
		if err != nil {
			for _, sub := range subs {
				sub.Unsubscribe()
				sub.Drain()
			}
			return err
		}
		subs = append(subs, sub)
	}
	t.subs = subs

	<-t.ctx.Done()

	return nil
}

func (t *NATS) Close() (merr error) {
	defer t.cancel()

	for _, sub := range t.subs {
		merr = multierr.Append(merr, sub.Unsubscribe())
		merr = multierr.Append(merr, sub.Drain())
	}

	return merr
}

func (t *NATS) handler(m *nats.Msg) {
	service := m.Header.Get("Service")
	function := m.Header.Get("Function")
	namespace := m.Header.Get("Namespace")
	id := m.Header.Get("ID")

	m.Header.Get("Content-Type")

	contentType := m.Header.Get("Content-Type")
	if contentType == "" {
		contentType = "application/json"
	}

	codec, ok := t.codecs[contentType]
	if !ok {
		header := make(nats.Header)
		header.Set("Status", strconv.Itoa(http.StatusUnsupportedMediaType))
		header.Set("Content-Type", "text/plain")

		message := fmt.Sprintf("%v: %s", ErrUnregisteredContentType, contentType)
		reply := nats.Msg{
			Header: header,
			Data:   []byte(message),
		}
		m.RespondMsg(&reply)
		return
	}

	ctx := context.Background()

	for _, filter := range t.filters {
		var err error
		if ctx, err = filter(ctx, m.Header); err != nil {
			t.handleError(err, codec, m, http.StatusInternalServerError)
			return
		}
	}

	requestBytes := m.Data

	var input interface{}
	if len(requestBytes) > 0 {
		if err := codec.Decode(requestBytes, &input); err != nil {
			t.handleError(err, codec, m, http.StatusInternalServerError)
			return
		}
	} else {
		input = map[string]interface{}{}
	}

	response, err := t.invoker(ctx, namespace, service, id, function, input)
	if err != nil {
		code := http.StatusInternalServerError
		if errors.Is(err, transport.ErrBadInput) {
			code = http.StatusBadRequest
		}
		t.handleError(err, codec, m, code)
		return
	}

	header := make(nats.Header)
	reply := nats.Msg{
		Header: header,
	}
	header.Set("Status", "200")
	header.Set("Content-Type", codec.ContentType())
	reply.Data, err = codec.Encode(response)
	if err != nil {
		t.handleError(err, codec, m, http.StatusInternalServerError)
		return
	}
	m.RespondMsg(&reply)
}

func (t *NATS) handleError(err error, codec channel.Codec, m *nats.Msg, status int) {
	errz := t.errorResolver(err)
	errz.Path = m.Subject

	header := make(nats.Header)
	header.Set("Status", strconv.Itoa(errz.Status))
	header.Set("Content-Type", codec.ContentType())

	payload, err := codec.Encode(errz)
	if err != nil {
		payload = []byte(errz.Message)
	}

	reply := nats.Msg{
		Header: header,
		Data:   payload,
	}
	m.RespondMsg(&reply)
}
