package dapr

import (
	"context"
	"fmt"

	proto "github.com/dapr/dapr/pkg/proto/components/v1"
	"go.opentelemetry.io/otel"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/codec"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/expr"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/resource"
	"github.com/nanobus/nanobus/telemetry/tracing"
)

type PublishConfig struct {
	// Resource is the name of the connection resource to use.
	Resource string `mapstructure:"resource" validate:"required"`
	// Topic is the name of the topic to publish to.
	Topic string `mapstructure:"topic" validate:"required"`
	// Codec is the configured codec to use for encoding the message.
	Codec string `mapstructure:"codec" validate:"required"`
	// CodecArgs are the arguments for the codec, if any.
	CodecArgs []interface{} `mapstructure:"codecArgs"`
	// Key is the optional value to use for the message key (is supported).
	Key *expr.ValueExpr `mapstructure:"key"`
	// Data is the input bindings sent
	Data *expr.DataExpr `mapstructure:"data"`
	// Metadata is the input binding metadata
	Metadata *expr.DataExpr `mapstructure:"metadata"`
	// PropogateTracing enables/disables propogating the distributed tracing context (e.g. W3C TraceContext standard).
	PropogateTracing bool `mapstructure:"propogateTracing"`
}

// Publish is the NamedLoader for the publish action.
func Publish() (string, actions.Loader) {
	return "@dapr/publish", PublishLoader
}

func PublishLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := PublishConfig{
		PropogateTracing: true,
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	var codecs codec.Codecs
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources,
		"codec:lookup", &codecs); err != nil {
		return nil, err
	}

	codec, ok := codecs[c.Codec]
	if !ok {
		return nil, fmt.Errorf("codec %q not found", c.Codec)
	}

	client, err := resource.Get[proto.PubSubClient](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return PublishAction(&c, codec, client), nil
}

func PublishAction(
	config *PublishConfig,
	codec codec.Codec,
	client proto.PubSubClient) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var err error

		var input interface{} = data["input"]
		if config.Data != nil {
			input, err = config.Data.Eval(data)
			if err != nil {
				return nil, err
			}
		}

		var key string
		if config.Key != nil {
			keyInt, err := config.Key.Eval(data)
			if err != nil {
				return nil, err
			}
			key = fmt.Sprintf("%v", keyInt)
		}

		// Propogate distributed tracing fields
		// per the the W3C TraceContext standard.
		if config.PropogateTracing {
			if m, ok := input.(map[string]interface{}); ok {
				otel.GetTextMapPropagator().Inject(ctx, tracing.MapCarrier(m))
			}
		}

		dataBytes, err := codec.Encode(input, config.CodecArgs...)
		if err != nil {
			return nil, err
		}

		metadata := map[string]string{}
		if key != "" {
			metadata["partitionKey"] = key
		}

		_, err = client.Publish(ctx, &proto.PublishRequest{
			Data:        dataBytes,
			PubsubName:  config.Resource,
			Topic:       config.Topic,
			Metadata:    metadata,
			ContentType: codec.ContentType(),
		})

		return nil, err
	}
}
