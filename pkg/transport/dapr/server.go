/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package dapr

import (
	"context"
	"fmt"
	"reflect"

	"github.com/dapr/go-sdk/service/common"
	daprd "github.com/dapr/go-sdk/service/grpc"
	"github.com/go-logr/logr"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/transport"
)

type Server struct {
	log     logr.Logger
	address string
	server  common.Service
	invoker transport.Invoker
	codecs  codec.Codecs
}

func DaprServerV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (transport.Transport, error) {
	// Defaults
	c := DaprServerV1Config{
		Address: ":19090",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var transportInvoker transport.Invoker
	var codecs codec.Codecs
	var logger logr.Logger
	if err := resolve.Resolve(resolver,
		"transport:invoker", &transportInvoker,
		"codec:lookup", &codecs,
		"system:logger", &logger); err != nil {
		return nil, err
	}

	server, err := daprd.NewService(c.Address)
	if err != nil {
		return nil, err
	}

	return NewServer(logger, server, transportInvoker, codecs, &c)
}

func NewServer(logger logr.Logger, server common.Service, transportInvoker transport.Invoker, codecs codec.Codecs, config *DaprServerV1Config) (*Server, error) {
	s := Server{
		log:     logger,
		address: config.Address,
		server:  server,
		invoker: transportInvoker,
		codecs:  codecs,
	}

	for _, subscription := range config.Subscriptions {
		logger.Info("Adding subscription", "subscription", subscription)
		if err := s.addSubscription(subscription); err != nil {
			return nil, err
		}
	}

	for _, binding := range config.Bindings {
		logger.Info("Adding binding", "binding", binding)
		if err := s.addBinding(binding); err != nil {
			return nil, err
		}
	}

	return &s, nil
}

func (s *Server) addSubscription(sub Subscription) error {
	codec, ok := s.codecs[string(sub.Codec)]
	if !ok {
		return fmt.Errorf("could not find codec %q", sub.Codec)
	}

	metadata := make(map[string]string, len(sub.Metadata)+1)
	for name, value := range sub.Metadata {
		metadata[name] = value
	}
	metadata["rawPayload"] = "true"

	return s.server.AddTopicEventHandler(&common.Subscription{
		PubsubName:             sub.Pubsub,
		Topic:                  sub.Topic,
		Metadata:               metadata,
		DisableTopicValidation: sub.DisableTopicValidation,
	}, func(ctx context.Context, e *common.TopicEvent) (retry bool, err error) {
		input, eventType, err := codec.Decode(e.RawData)
		if err != nil {
			return false, err
		}

		var h handler.Handler
		var handlerFound bool

		// Default handler
		if sub.Handler != nil {
			h = *sub.Handler
			handlerFound = true
		}

		// Handler for event type
		if eventType != "" && sub.Types != nil {
			if handler, ok := sub.Types[eventType]; ok {
				h = handler
				handlerFound = true
			}
		}

		if handlerFound {
			_, err = s.invoker(ctx, h, "", input, transport.BypassAuthorization)
		}

		return false, err
	})
}

func (s *Server) addBinding(binding Binding) error {
	codec, ok := s.codecs[string(binding.Codec)]
	if !ok {
		return fmt.Errorf("could not find codec %q", binding.Codec)
	}

	return s.server.AddBindingInvocationHandler(binding.Name, func(ctx context.Context, in *common.BindingEvent) (out []byte, err error) {
		input, _, err := codec.Decode(in.Data)
		if err != nil {
			return nil, err
		}

		output, err := s.invoker(ctx, binding.Handler, "", input, transport.BypassAuthorization)
		if err != nil {
			return nil, err
		}

		if !isNil(output) {
			out, err = codec.Encode(output)
		}

		return out, err
	})
}

func (s *Server) Listen() error {
	s.log.Info("Dapr server listening", "address", s.address)
	return s.server.Start()
}

func (s *Server) Close() (err error) {
	return s.server.GracefulStop()
}

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
}
