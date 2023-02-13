package resiliency

import (
	"time"

	"github.com/nanobus/nanobus/pkg/resiliency/breaker"
	"github.com/nanobus/nanobus/pkg/resiliency/retry"
)

type Policies struct {
	Timeouts        map[string]time.Duration
	Retries         map[string]*retry.Config
	CircuitBreakers map[string]*breaker.CircuitBreaker
}
