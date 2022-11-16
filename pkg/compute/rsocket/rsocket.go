/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package rsocket

// import (
// 	"context"
// 	"errors"
// 	"fmt"
// 	"io"
// 	"os"
// 	"strconv"
// 	"strings"
// 	"sync"

// 	"github.com/go-logr/logr"
// 	"github.com/rsocket/rsocket-go"
// 	"github.com/rsocket/rsocket-go/payload"
// 	"github.com/rsocket/rsocket-go/rx/flux"
// 	"github.com/rsocket/rsocket-go/rx/mono"

// 	"github.com/nanobus/nanobus/pkg/channel"
// 	"github.com/nanobus/nanobus/pkg/channel/metadata"
// 	"github.com/nanobus/nanobus/pkg/compute"
// 	"github.com/nanobus/nanobus/pkg/config"
// 	"github.com/nanobus/nanobus/pkg/resolve"
// 	"github.com/nanobus/nanobus/pkg/stream"
// 	"github.com/nanobus/nanobus/pkg/structerror"
// )

// var ErrInvalidURISyntax = errors.New("invalid invocation URI syntax")
// var ErrNotConnected = errors.New("application is not connected")

// type RSocketConfig struct {
// 	BasePath string `mapstructure:"basePath"`
// 	Host     string `mapstructure:"host"`
// 	Port     int    `mapstructure:"port"`
// }

// // RSocket is the NamedLoader for the RSocket compute.
// func RSocket() (string, compute.Loader) {
// 	return "rsocket", RSocketLoader
// }

// func RSocketLoader(with interface{}, resolver resolve.ResolveAs) (*compute.Compute, error) {
// 	port, err := strconv.Atoi(defaultStringValue(os.Getenv("RSOCKET_PORT"), "7878"))
// 	if err != nil {
// 		return nil, err
// 	}
// 	c := RSocketConfig{
// 		BasePath: defaultStringValue(os.Getenv("RSOCKET_BASEPATH"), "/"),
// 		Host:     defaultStringValue(os.Getenv("RSOCKET_HOST"), "127.0.0.1"),
// 		Port:     port,
// 	}
// 	if err := config.Decode(with, &c); err != nil {
// 		return nil, err
// 	}

// 	var msgpackcodec channel.Codec
// 	var busInvoker compute.BusInvoker
// 	var stateInvoker compute.StateInvoker
// 	var log logr.Logger
// 	if err := resolve.Resolve(resolver,
// 		"codec:msgpack", &msgpackcodec,
// 		"bus:invoker", &busInvoker,
// 		"state:invoker", &stateInvoker,
// 		"system:logger", &log); err != nil {
// 		return nil, err
// 	}

// 	socket := newSocket(c.BasePath, msgpackcodec, busInvoker, stateInvoker)
// 	ctx, cancel := context.WithCancel(context.Background())
// 	tp := rsocket.TCPServer().SetHostAndPort(c.Host, c.Port).Build()
// 	start := rsocket.Receive().
// 		OnStart(func() {
// 			log.Info("RSocket server started", "host", c.Host, "port", c.Port)
// 		}).
// 		Acceptor(func(ctx context.Context, setup payload.SetupPayload, sendingSocket rsocket.CloseableRSocket) (rsocket.RSocket, error) {
// 			socket.setSendingSocket(ctx, sendingSocket)
// 			return socket.responder(), nil
// 		}).
// 		Transport(tp)

// 	invoker := channel.NewInvoker(socket.Invoke, socket.InvokeStream, msgpackcodec)

// 	return &compute.Compute{
// 		Invoker: invoker,
// 		Start: func() error {
// 			err := start.Serve(ctx)
// 			if err != nil {
// 				return err
// 			}
// 			<-ctx.Done()
// 			return nil
// 		},
// 		WaitUntilShutdown: func() error {
// 			//conn.WaitUntilShutdown()
// 			<-ctx.Done()
// 			return nil
// 		},
// 		Close: func() error {
// 			cancel()
// 			return nil
// 		},
// 		Environ: func() []string {
// 			return []string{
// 				fmt.Sprintf("RSOCKET_HOST=%s", c.Host),
// 				fmt.Sprintf("RSOCKET_PORT=%d", c.Port),
// 			}
// 		},
// 	}, nil
// }

// type Socket struct {
// 	basePath string
// 	ctx      context.Context
// 	codec    channel.Codec
// 	socket   rsocket.CloseableRSocket
// 	close    chan struct{}

// 	busInvoker   compute.BusInvoker
// 	stateInvoker compute.StateInvoker
// }

// func newSocket(basePath string,
// 	codec channel.Codec,
// 	busInvoker compute.BusInvoker,
// 	stateInvoker compute.StateInvoker) *Socket {
// 	return &Socket{
// 		ctx:          context.Background(),
// 		basePath:     basePath,
// 		codec:        codec,
// 		close:        make(chan struct{}, 10),
// 		busInvoker:   busInvoker,
// 		stateInvoker: stateInvoker,
// 	}
// }

// func (s *Socket) setSendingSocket(ctx context.Context, socket rsocket.CloseableRSocket) {
// 	s.ctx = ctx
// 	s.socket = socket
// 	socket.OnClose(func(e error) {
// 		s.close <- struct{}{}
// 		s.socket = nil
// 	})
// }

// func (s *Socket) WaitUntilShutdown() {
// 	if s.socket == nil {
// 		return
// 	}
// 	<-s.close
// }

// func (s *Socket) responder() rsocket.RSocket {
// 	return rsocket.NewAbstractSocket(
// 		rsocket.RequestResponse(func(request payload.Payload) mono.Mono {
// 			p, err := s.getPath(request)
// 			if err != nil {
// 				fmt.Println(err)
// 				return mono.Error(err)
// 			}

// 			data := request.Data()

// 			return mono.Create(func(ctx context.Context, sink mono.Sink) {
// 				if p.namespace == "nanobus" && p.service == "state" && p.function == "get" {
// 					type Args struct {
// 						Namespace string `json:"namespace" msgpack:"namespace"`
// 						ID        string `json:"id" msgpack:"id"`
// 						Key       string `json:"key" msgpack:"key"`
// 					}

// 					var args Args
// 					err := s.codec.Decode(data, &args)
// 					if err != nil {
// 						fmt.Println(err)
// 						sink.Error(err)
// 						return
// 					}
// 					output, err := s.stateInvoker(s.ctx, args.Namespace, args.ID, args.Key)
// 					if err != nil {
// 						fmt.Println(err)
// 						sink.Error(err)
// 						return
// 					}

// 					sink.Success(payload.New(output, nil))
// 					return
// 				}

// 				var input interface{}
// 				if len(data) > 0 {
// 					if err := s.codec.Decode(data, &input); err != nil {
// 						fmt.Println(err)
// 						sink.Error(err)
// 						return
// 					}
// 				}

// 				output, err := s.busInvoker(s.ctx, p.namespace, p.service, p.function, input)
// 				if err != nil {
// 					fmt.Println(err)
// 					sink.Error(err)
// 					return
// 				}

// 				var outputBytes []byte
// 				if output != nil {
// 					if outputBytes, err = s.codec.Encode(output); err != nil {
// 						fmt.Println(err)
// 						sink.Error(err)
// 						return
// 					}
// 				}
// 				if err != nil {
// 					fmt.Println(err)
// 					sink.Error(err)
// 					return
// 				}
// 				sink.Success(payload.New(outputBytes, nil))
// 			})
// 		}),
// 		rsocket.RequestStream(func(request payload.Payload) flux.Flux {
// 			p, err := s.getPath(request)
// 			if err != nil {
// 				fmt.Println(err)
// 				return flux.Error(err)
// 			}

// 			data := request.Data()
// 			var input interface{}
// 			if len(data) > 0 {
// 				if err := s.codec.Decode(data, &input); err != nil {
// 					fmt.Println(err)
// 					return flux.Error(err)
// 				}
// 			}

// 			return flux.Create(func(ctx context.Context, emitter flux.Sink) {
// 				// Create stream and add to context
// 				str := CodecStream{
// 					ctx:   ctx,
// 					codec: s.codec,
// 					sink:  emitter,
// 					md:    p.md,
// 				}
// 				defer str.Close()
// 				ctx = stream.NewContext(s.ctx, &str)

// 				_, err := s.busInvoker(ctx, p.namespace, p.service, p.function, input)
// 				if err != nil {
// 					fmt.Println(err)
// 					emitter.Error(err)
// 					return
// 				}
// 			})
// 		}),
// 		rsocket.RequestChannel(func(payloads flux.Flux) flux.Flux {
// 			return flux.Create(func(ctx context.Context, emitter flux.Sink) {
// 				c := make(chan payload.Payload, 100)
// 				e := make(chan error, 1)
// 				payloads.DoOnNext(func(input payload.Payload) error {
// 					c <- payload.Clone(input)
// 					return nil
// 				}).DoOnError(func(err error) {
// 					e <- err
// 				}).DoOnComplete(func() {
// 					close(c)
// 				}).Subscribe(ctx)

// 				var request payload.Payload
// 				var err error
// 				select {
// 				case request = <-c:
// 				case err = <-e:
// 				}
// 				if err != nil {
// 					fmt.Println(err)
// 					emitter.Error(err)
// 					return
// 				}
// 				if request == nil {
// 					return
// 				}

// 				data := request.Data()
// 				p, err := s.getPath(request)
// 				if err != nil {
// 					fmt.Println(err)
// 					emitter.Error(err)
// 					return
// 				}

// 				var input interface{}
// 				if len(data) > 0 {
// 					if err := s.codec.Decode(data, &input); err != nil {
// 						fmt.Println(err)
// 						emitter.Error(err)
// 						return
// 					}
// 				}

// 				// Create stream and add to context
// 				str := CodecStream{
// 					ctx:   ctx,
// 					codec: s.codec,
// 					c:     c,
// 					e:     e,
// 					sink:  emitter,
// 					md:    p.md,
// 				}
// 				defer str.Close()
// 				ctx = stream.NewContext(s.ctx, &str)

// 				_, err = s.busInvoker(ctx, p.namespace, p.service, p.function, input)
// 				if err != nil {
// 					fmt.Println(err)
// 					emitter.Error(err)
// 					return
// 				}
// 			})
// 		}),
// 	)
// }

// func (s *Socket) Invoke(ctx context.Context, receiver channel.Receiver, data []byte) ([]byte, error) {
// 	socket := s.socket
// 	if socket == nil {
// 		return nil, ErrNotConnected
// 	}
// 	path := s.basePath + receiver.Namespace + "/" + receiver.Operation
// 	md := metadata.MD{
// 		":path": []string{path},
// 	}
// 	if receiver.EntityID != "" {
// 		md[":id"] = []string{receiver.EntityID}
// 	}
// 	mdBytes, err := s.codec.Encode(md)
// 	if err != nil {
// 		return nil, err
// 	}
// 	resp, err := socket.RequestResponse(payload.New(data, mdBytes)).Block(ctx)
// 	if err != nil {
// 		if appErr, ok := err.(rsocket.Error); ok {
// 			return nil, structerror.Parse(string(appErr.ErrorData()))
// 		}
// 		return nil, err
// 	}

// 	return resp.Data(), nil
// }

// func (s *Socket) InvokeStream(ctx context.Context, receiver channel.Receiver) (channel.Streamer, error) {
// 	path := s.basePath + receiver.Namespace + "/" + receiver.Operation
// 	md := metadata.MD{
// 		":path": []string{path},
// 	}
// 	if receiver.EntityID != "" {
// 		md[":id"] = []string{receiver.EntityID}
// 	}
// 	mdBytes, err := s.codec.Encode(md)
// 	if err != nil {
// 		return nil, err
// 	}
// 	pl := payload.New([]byte{0, 0, 0}, mdBytes)

// 	socket := s.socket
// 	if socket == nil {
// 		return nil, ErrNotConnected
// 	}
// 	processor := flux.CreateProcessor()
// 	f := socket.RequestStream(pl)
// 	c := make(chan payload.Payload, 100)
// 	e := make(chan error, 1)
// 	f.DoOnNext(func(input payload.Payload) error {
// 		c <- payload.Clone(input)
// 		return nil
// 	}).DoOnError(func(err error) {
// 		if appErr, ok := err.(rsocket.Error); ok {
// 			err = structerror.Parse(string(appErr.ErrorData()))
// 		}
// 		e <- err
// 	}).DoOnComplete(func() {
// 		close(c)
// 	}).Subscribe(ctx)

// 	stream := Stream{
// 		ctx:  ctx,
// 		sink: processor,
// 		c:    c,
// 		e:    e,
// 	}
// 	return &stream, nil
// }

// type Stream struct {
// 	server *Socket
// 	ctx    context.Context
// 	sink   flux.Sink
// 	c      <-chan payload.Payload
// 	e      <-chan error

// 	sendMd metadata.MD
// 	md     metadata.MD
// }

// func (s *Stream) SendMetadata(md metadata.MD, end ...bool) error {
// 	var endVal bool
// 	if len(end) > 0 {
// 		endVal = end[0]
// 	}
// 	if endVal {
// 		mdBytes, err := s.server.codec.Encode(md)
// 		if err != nil {
// 			return err
// 		}
// 		s.sink.Next(payload.New(nil, mdBytes))
// 		return nil
// 	}

// 	s.sendMd = md
// 	return nil
// }

// func (s *Stream) SendData(data []byte, end ...bool) error {
// 	var endVal bool
// 	if len(end) > 0 {
// 		endVal = end[0]
// 	}

// 	var mdBytes []byte
// 	if s.sendMd != nil {
// 		var err error
// 		mdBytes, err = s.server.codec.Encode(s.sendMd)
// 		if err != nil {
// 			return err
// 		}
// 		s.sendMd = nil
// 	}
// 	s.sink.Next(payload.New(data, mdBytes))
// 	if endVal {
// 		s.sink.Complete()
// 	}
// 	return nil
// }

// func (s *Stream) Close() error {
// 	s.sink.Complete()
// 	return nil
// }

// func (s *Stream) Metadata() metadata.MD { return s.md }
// func (s *Stream) RecvData() ([]byte, error) {
// 	select {
// 	case payload := <-s.c:
// 		if payload == nil {
// 			return nil, io.EOF
// 		}
// 		if mdBytes, ok := payload.Metadata(); ok {
// 			var md metadata.MD
// 			if err := s.server.codec.Decode(mdBytes, &md); err != nil {
// 				return nil, err
// 			}
// 			s.md = md
// 		}

// 		return payload.Data(), nil
// 	case err := <-s.e:
// 		return nil, err
// 	case <-s.ctx.Done():
// 		return nil, context.Canceled
// 	}
// }

// type path struct {
// 	namespace string
// 	service   string
// 	function  string
// 	md        metadata.MD
// }

// func (s *Socket) getPath(request payload.Payload) (p path, err error) {
// 	var path string
// 	if mdBytes, ok := request.Metadata(); ok {
// 		var md metadata.MD
// 		if err := s.codec.Decode(mdBytes, &md); err == nil {
// 			path, _ = md.Scalar(":path")
// 		}
// 		p.md = md
// 	}
// 	path = strings.TrimPrefix(path, "/")
// 	parts := strings.Split(path, "/")
// 	if len(parts) != 2 {
// 		return p, ErrInvalidURISyntax
// 	}

// 	p.namespace = parts[0]
// 	p.function = parts[1]

// 	lastDot := strings.LastIndexAny(p.namespace, ".:")
// 	if lastDot < 0 {
// 		return p, ErrInvalidURISyntax
// 	}
// 	p.service = p.namespace[lastDot+1:]
// 	p.namespace = p.namespace[:lastDot]

// 	return p, nil
// }

// type CodecStream struct {
// 	ctx   context.Context
// 	c     <-chan payload.Payload
// 	e     <-chan error
// 	sink  flux.Sink
// 	codec channel.Codec
// 	md    metadata.MD

// 	sendMd metadata.MD
// 	close  sync.Once
// }

// func (s *CodecStream) Metadata() metadata.MD {
// 	return s.md
// }

// type RefCounter interface {
// 	IncRef() int32
// 	Release()
// }

// func (s *CodecStream) RecvData(dst interface{}) error {
// 	select {
// 	case payload, ok := <-s.c:
// 		if !ok || payload == nil {
// 			return io.EOF
// 		}
// 		if mdBytes, ok := payload.Metadata(); ok {
// 			var md metadata.MD
// 			if err := s.codec.Decode(mdBytes, &md); err != nil {
// 				return err
// 			}
// 			s.md = md
// 		}

// 		data := payload.Data()
// 		if len(data) > 0 {
// 			var md metadata.MD
// 			if err := s.codec.Decode(data, dst); err != nil {
// 				return err
// 			}
// 			s.md = md
// 		}

// 		return nil
// 	case err := <-s.e:
// 		return err
// 	case <-s.ctx.Done():
// 		return context.Canceled
// 	}
// }

// func (s *CodecStream) SendHeaders(md metadata.MD, end ...bool) error {
// 	var endVal bool
// 	if len(end) > 0 {
// 		endVal = end[0]
// 	}
// 	if endVal {
// 		mdBytes, err := s.codec.Encode(md)
// 		if err != nil {
// 			return err
// 		}
// 		s.sink.Next(payload.New(nil, mdBytes))
// 		s.Close()
// 		return nil
// 	}

// 	s.sendMd = md
// 	return nil
// }

// func (s *CodecStream) SendData(v interface{}, end ...bool) error {
// 	var endVal bool
// 	if len(end) > 0 {
// 		endVal = end[0]
// 	}

// 	var mdBytes []byte
// 	if s.sendMd != nil {
// 		var err error
// 		mdBytes, err = s.codec.Encode(s.sendMd)
// 		if err != nil {
// 			return err
// 		}
// 		s.sendMd = nil
// 	}

// 	data, err := s.codec.Encode(v)
// 	if err != nil {
// 		return err
// 	}

// 	s.sink.Next(payload.New(data, mdBytes))
// 	if endVal {
// 		s.Close()
// 	}
// 	return nil
// }

// func (s *CodecStream) Close() error {
// 	s.close.Do(func() {
// 		if s.sink != nil {
// 			s.sink.Complete()
// 		}
// 	})
// 	return nil
// }

// func (s *CodecStream) SendUnary(md metadata.MD, v interface{}) (err error) {
// 	var mdBytes []byte
// 	if md != nil {
// 		mdBytes, err = s.codec.Encode(md)
// 		if err != nil {
// 			return err
// 		}
// 	}

// 	var valBytes []byte
// 	switch v := v.(type) {
// 	case nil:
// 		// Do nothing
// 	case []byte:
// 		valBytes = v
// 	default:
// 		var err error
// 		valBytes, err = s.codec.Encode(v)
// 		if err != nil {
// 			return fmt.Errorf("could not marshal value to send: %w", err)
// 		}
// 	}

// 	s.sink.Next(payload.New(valBytes, mdBytes))

// 	return nil
// }

// func (s *CodecStream) SendRequest(path string, v interface{}) error {
// 	return s.SendUnary(metadata.MD{
// 		":path":        []string{path},
// 		"content-type": []string{s.codec.ContentType()},
// 	}, v)
// }

// func (s *CodecStream) SendReply(v interface{}) error {
// 	return s.SendUnary(metadata.MD{
// 		":status":      []string{"200"},
// 		"content-type": []string{s.codec.ContentType()},
// 	}, v)
// }

// func (s *CodecStream) SendError(err error) error {
// 	msg := err.Error()
// 	return s.SendUnary(metadata.MD{
// 		":status":      []string{"500"},        //strconv.Itoa(e.Status)
// 		"content-type": []string{"text/plain"}, //s.codec.ContentType()
// 	}, []byte(msg))
// }

// func defaultStringValue(val string, defaultValue string) string {
// 	if val == "" {
// 		return defaultValue
// 	}
// 	return val
// }
