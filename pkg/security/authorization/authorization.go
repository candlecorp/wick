package authorization

import (
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/security/claims"
)

type Rule interface {
	Check(c claims.Claims) error
}

type unauthenticated struct{}

func (unauthenticated) Check(c claims.Claims) error {
	return nil
}

var Unauthenticated = unauthenticated{}

type Basic struct {
	has    []string
	checks map[string]any
	rules  []Rule
}

func NewBasic(
	has []string,
	checks map[string]any,
	rules ...Rule) *Basic {
	return &Basic{
		has:    has,
		checks: checks,
		rules:  rules,
	}
}

func (b *Basic) Check(c claims.Claims) error {
	if len(c) == 0 {
		return errorz.Return("unauthenticated", errorz.Metadata{})
	}

	for _, claim := range b.has {
		if _, ok := c[claim]; !ok {
			return errorz.Return("permission_denied", errorz.Metadata{
				"claim": claim,
			})
		}
	}

	for claim, value := range b.checks {
		v := c[claim]
		if v != value {
			return errorz.Return("permission_denied", errorz.Metadata{
				"claim": claim,
				"want":  value,
			})
		}
	}

	for _, rule := range b.rules {
		if err := rule.Check(c); err != nil {
			return err
		}
	}

	return nil
}
