package resiliency_test

import (
	"context"
	"testing"
	"time"

	"github.com/go-logr/logr"
	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/resiliency"
	"github.com/nanobus/nanobus/resiliency/breaker"
	"github.com/nanobus/nanobus/resiliency/retry"
)

func TestPolicy(t *testing.T) {
	retryValue := retry.DefaultConfig
	cbValue := breaker.CircuitBreaker{
		Name:     "test",
		Interval: 10 * time.Millisecond,
		Timeout:  10 * time.Millisecond,
	}
	log := logr.Discard()
	cbValue.Initialize(log)
	tests := map[string]struct {
		t  time.Duration
		r  *retry.Config
		cb *breaker.CircuitBreaker
	}{
		"empty": {},
		"all": {
			t:  10 * time.Millisecond,
			r:  &retryValue,
			cb: &cbValue,
		},
	}

	ctx := context.Background()
	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			called := false
			fn := func(ctx context.Context) error {
				called = true

				return nil
			}
			policy := resiliency.Policy(logr.Discard(), name, tt.t, tt.r, tt.cb)
			policy(ctx, fn)
			assert.True(t, called)
		})
	}
}
