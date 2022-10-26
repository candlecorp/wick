package core

import (
	"context"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/resolve"
)

type CallPipelineConfig struct {
	// Name is the name of the pipeline to call.
	Name string `mapstructure:"name" validate:"required"`
}

// Route is the NamedLoader for the filter action.
func CallPipeline() (string, actions.Loader) {
	return "call_pipeline", CallPipelineLoader
}

func CallPipelineLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c CallPipelineConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var processor Processor
	if err := resolve.Resolve(resolver,
		"system:processor", &processor); err != nil {
		return nil, err
	}

	return CallPipelineAction(&c, processor), nil
}

func CallPipelineAction(
	config *CallPipelineConfig, processor Processor) actions.Action {
	return func(ctx context.Context, data actions.Data) (output interface{}, err error) {
		return processor.Pipeline(ctx, config.Name, data)
	}
}
