package dapr

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/cenkalti/backoff/v4"
	proto "github.com/dapr/dapr/pkg/proto/components/v1"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/expr"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/resource"
)

type SetStateConfig struct {
	// Resource is name of state store to invoke.
	Resource string         `mapstructure:"resource" validate:"required"`
	Items    []SetStateItem `mapstructure:"items" validate:"required"`
}

type SetStateItem struct {
	// Key is the expression to evaluate the key to save.
	Key *expr.ValueExpr `mapstructure:"key" validate:"required"`
	// ForEach is an option expression to evaluate a
	ForEach *expr.ValueExpr `mapstructure:"forEach"`
	// Value is the optional data expression to tranform the data to set.
	Value *expr.DataExpr `mapstructure:"value"`
	// Metadata is the optional data expression for the key's metadata.
	Metadata *expr.DataExpr `mapstructure:"metadata"`
}

// SetState is the NamedLoader for the Dapr get state operation
func SetState() (string, actions.Loader) {
	return "@dapr/set_state", SetStateLoader
}

func SetStateLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c SetStateConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	client, err := resource.Get[proto.StateStoreClient](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return SetStateAction(client, &c), nil
}

func SetStateAction(
	client proto.StateStoreClient,
	config *SetStateConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		_r := [25]*proto.SetRequest{}
		r := _r[0:0]

		for i := range config.Items {
			configItems := &config.Items[i]
			var items []interface{}
			if configItems.ForEach != nil {
				itemsInt, err := configItems.ForEach.Eval(data)
				if err != nil {
					return nil, backoff.Permanent(fmt.Errorf("could not evaluate data: %w", err))
				}
				var ok bool
				if items, ok = itemsInt.([]interface{}); ok {
					return nil, backoff.Permanent(fmt.Errorf("forEach expression %q did not return a slice of items", configItems.ForEach.Expr()))
				}
			}

			if items == nil {
				it, err := createSetItem(data, nil, configItems)
				if err != nil {
					return nil, err
				}

				r = append(r, it)
			} else {
				for _, item := range items {
					it, err := createSetItem(data, item, configItems)
					if err != nil {
						return nil, err
					}

					r = append(r, it)
				}
			}
		}

		_, err := client.BulkSet(ctx, &proto.BulkSetRequest{
			Items: r,
		})

		return nil, err
	}
}

func createSetItem(
	data actions.Data,
	item interface{},
	config *SetStateItem) (it *proto.SetRequest, err error) {
	variables := make(map[string]interface{}, len(data)+1)
	for k, v := range data {
		variables[k] = v
	}
	variables["item"] = item

	var value interface{} = variables["input"]
	it = &proto.SetRequest{}
	keyInt, err := config.Key.Eval(variables)
	if err != nil {
		return it, backoff.Permanent(fmt.Errorf("could not evaluate key: %w", err))
	}
	it.Key = fmt.Sprintf("%v", keyInt)

	if config.Value != nil {
		if value, err = config.Value.Eval(variables); err != nil {
			return it, backoff.Permanent(fmt.Errorf("could not evaluate value: %w", err))
		}
	}
	if config.Metadata != nil {
		if it.Metadata, err = config.Metadata.EvalMap(variables); err != nil {
			return it, backoff.Permanent(fmt.Errorf("could not evaluate metadata: %w", err))
		}
	}

	// TODO: ues codec
	jsonBytes, err := json.Marshal(value)
	if err != nil {
		return it, backoff.Permanent(fmt.Errorf("could not serialize value: %w", err))
	}
	it.Value = jsonBytes

	return it, nil
}
