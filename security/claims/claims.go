package claims

import (
	"context"
)

type Claims map[string]interface{}

func Combine(claimsList ...Claims) Claims {
	merged := make(Claims)

	for _, claims := range claimsList {
		if claims == nil {
			continue
		}

		for k, v := range claims {
			merged[k] = v
		}
	}

	return merged
}

type claimsKey struct{}

func FromContext(ctx context.Context) Claims {
	v := ctx.Value(claimsKey{})
	if v == nil {
		return Claims{}
	}
	c, _ := v.(Claims)
	if c == nil {
		return Claims{}
	}

	return c
}

func ToContext(ctx context.Context, claims Claims) context.Context {
	return context.WithValue(ctx, claimsKey{}, claims)
}
