package core

import (
	"context"
	"fmt"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/expr"
	"github.com/nanobus/nanobus/resolve"
)

type FilterConfig struct {
	// Condition is the predicate expression for filtering.
	Condition *expr.ValueExpr `mapstructure:"condition" validate:"required"`
}

// Filter is the NamedLoader for the filter action.
func Filter() (string, actions.Loader) {
	return "filter", FilterLoader
}

func FilterLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c FilterConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return FilterAction(&c), nil
}

func FilterAction(
	config *FilterConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		resultInt, err := config.Condition.Eval(data)
		if err != nil {
			return nil, err
		}

		result, ok := resultInt.(bool)
		if !ok {
			return nil, fmt.Errorf("expression %q did not evaluate a boolean", config.Condition.Expr())
		}

		if !result {
			return nil, actions.Stop()
		}

		return nil, nil
	}
}
