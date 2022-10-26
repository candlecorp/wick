package core

import (
	"context"
	"fmt"

	"github.com/jmespath/go-jmespath"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/expr"
	"github.com/nanobus/nanobus/resolve"
)

type JMESPathConfig struct {
	// Path is the predicate expression for filtering.
	Path string `mapstructure:"path" validate:"required"`
	// Data is the optional data expression to pass to jq.
	Data *expr.DataExpr `mapstructure:"data"`
	// Var, if set, is the variable that is set with the result.
	Var string `mapstructure:"var"`
}

// JMESPath is the NamedLoader for the jmespath action.
func JMESPath() (string, actions.Loader) {
	return "jmespath", JMESPathLoader
}

func JMESPathLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c JMESPathConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	jp, err := jmespath.Compile(c.Path)
	if err != nil {
		return nil, fmt.Errorf("error compiling jmespath query: %w", err)
	}

	return JMESPathAction(&c, jp), nil
}

func JMESPathAction(
	config *JMESPathConfig, jp *jmespath.JMESPath) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var in interface{} = map[string]interface{}(data)
		if config.Data != nil {
			var err error
			in, err = config.Data.Eval(data)
			if err != nil {
				return nil, err
			}
		}

		result, err := safeSearch(in, jp)
		if err != nil {
			return nil, err
		}

		if config.Var != "" {
			data[config.Var] = result
		}

		return result, nil
	}
}

func safeSearch(v interface{}, j *jmespath.JMESPath) (result interface{}, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("jmespath panic: %v", r)
		}
	}()
	return j.Search(v)
}
