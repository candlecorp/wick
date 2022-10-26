package core

import (
	"context"
	"fmt"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/errorz"
	"github.com/nanobus/nanobus/expr"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/security/claims"
)

type AuthorizeConfig struct {
	// Condition is the predicate expression for authorization.
	Condition *expr.ValueExpr        `mapstructure:"condition"`
	Has       []string               `mapstructure:"has"`
	Check     map[string]interface{} `mapstructure:"check"`
	Error     string                 `mapstructure:"error"`
}

// Authorize is the NamedLoader for the log action.
func Authorize() (string, actions.Loader) {
	return "authorize", AuthorizeLoader
}

func AuthorizeLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := AuthorizeConfig{
		Error: "permission_denied",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return AuthorizeAction(&c), nil
}

func AuthorizeAction(
	config *AuthorizeConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		if config.Condition != nil {
			resultInt, err := config.Condition.Eval(data)
			if err != nil {
				return nil, err
			}

			result, ok := resultInt.(bool)
			if !ok {
				return nil, fmt.Errorf("expression %q did not evaluate a boolean", config.Condition.Expr())
			}

			if !result {
				return nil, errorz.Return(config.Error, errorz.Metadata{
					"expr": config.Condition.Expr(),
				})
			}
		}

		claimsMap := claims.FromContext(ctx)

		for _, claim := range config.Has {
			if _, ok := claimsMap[claim]; !ok {
				return nil, errorz.Return(config.Error, errorz.Metadata{
					"claim": claim,
				})
			}
		}

		for claim, value := range config.Check {
			v := claimsMap[claim]
			if v != value {
				return nil, errorz.Return(config.Error, errorz.Metadata{
					"claim": claim,
					"want":  value,
				})
			}
		}

		return nil, nil
	}
}
