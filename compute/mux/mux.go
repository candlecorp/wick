package mux

// import (
// 	"context"
// 	"errors"
// 	"os"

// 	"github.com/nanobus/nanobus/channel"
// 	msgpack_codec "github.com/nanobus/nanobus/channel/codecs/msgpack"
// 	transport_mux "github.com/nanobus/nanobus/channel/transports/mux"

// 	"github.com/nanobus/nanobus/compute"
// 	"github.com/nanobus/nanobus/config"
// 	"github.com/nanobus/nanobus/errorz"
// 	"github.com/nanobus/nanobus/resolve"
// )

// const defaultInvokeURL = "http://127.0.0.1:9000"

// type MuxConfig struct {
// 	BaseURL string `mapstructure:"baseUrl"`
// }

// // Mux is the NamedLoader for the mux compute.
// func Mux() (string, compute.Loader) {
// 	return "mux", MuxLoader
// }

// func MuxLoader(with interface{}, resolver resolve.ResolveAs) (*compute.Compute, error) {
// 	baseURL := os.Getenv("APP_URL")
// 	if baseURL == "" {
// 		baseURL = defaultInvokeURL
// 	}
// 	c := MuxConfig{
// 		BaseURL: baseURL,
// 	}
// 	if err := config.Decode(with, &c); err != nil {
// 		return nil, err
// 	}

// 	msgpackcodec := msgpack_codec.New()
// 	m := transport_mux.New(c.BaseURL, msgpackcodec.ContentType())
// 	invokeStream := func(ctx context.Context, receiver channel.Receiver) (channel.Streamer, error) {
// 		return nil, errors.New(errorz.Unimplemented.String())
// 	}
// 	invoker := channel.NewInvoker(m.Invoke, invokeStream, msgpackcodec)
// 	done := make(chan struct{}, 1)

// 	return &compute.Compute{
// 		Invoker: invoker,
// 		Start:   func() error { return nil },
// 		WaitUntilShutdown: func() error {
// 			<-done
// 			return nil
// 		},
// 		Close: func() error {
// 			close(done)
// 			return nil
// 		},
// 	}, nil
// }
