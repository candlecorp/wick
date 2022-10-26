package dapr

import (
	"context"
	"encoding/json"

	proto "github.com/dapr/dapr/pkg/proto/components/v1"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/coalesce"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/expr"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/resource"
)

type InvokeBindingConfig struct {
	// Resource is name of binding to invoke.
	Resource string `mapstructure:"resource" validate:"required"`
	// Operation is the name of the operation type for the binding to invoke
	Operation string `mapstructure:"operation" validate:"required"`
	// Data is the input bindings sent
	Data *expr.DataExpr `mapstructure:"data"`
	// Metadata is the input binding metadata
	Metadata *expr.DataExpr `mapstructure:"metadata"`
}

// InvokeBinding is the NamedLoader for Dapr output bindings
func InvokeBinding() (string, actions.Loader) {
	return "@dapr/invoke_binding", InvokeBindingLoader
}

func InvokeBindingLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c InvokeBindingConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	client, err := resource.Get[proto.OutputBindingClient](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return InvokeBindingAction(client, &c), nil
}

func InvokeBindingAction(
	client proto.OutputBindingClient,
	config *InvokeBindingConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var bindingData interface{}
		r := proto.InvokeRequest{
			Operation: config.Operation,
		}

		var err error
		if config.Data != nil {
			if bindingData, err = config.Data.Eval(data); err != nil {
				return nil, err
			}
		}
		if config.Metadata != nil {
			if r.Metadata, err = config.Metadata.EvalMap(data); err != nil {
				return nil, err
			}
		}

		// TODO: multi-codec support
		if r.Data, err = json.Marshal(bindingData); err != nil {
			return nil, err
		}

		resp, err := client.Invoke(ctx, &r)
		if err != nil {
			return nil, err
		}

		var response interface{}
		if len(resp.Data) > 0 {
			err = coalesce.JSONUnmarshal(resp.Data, &response)
		}

		return response, err
	}
}
