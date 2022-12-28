package runtime

import (
	"errors"
	"time"

	"github.com/nanobus/nanobus/pkg/resiliency/retry"
)

// ConvertBackoffConfig decodes a Go struct into a `retry.Config`.
func ConvertBackoffConfig(b Backoff) (retry.Config, error) {
	def := retry.DefaultConfig
	if b.Constant != nil {
		def.Policy = retry.PolicyConstant
		def.Duration = time.Duration(b.Constant.Duration)
		if b.Constant.MaxRetries != nil {
			def.MaxRetries = int64(*b.Constant.MaxRetries)
		}
	} else if b.Exponential != nil {
		def.Policy = retry.PolicyExponential
		def.InitialInterval = time.Duration(b.Exponential.InitialInterval)
		def.MaxInterval = time.Duration(b.Exponential.MaxInterval)
		def.MaxElapsedTime = time.Duration(b.Exponential.MaxElapsedTime)

		if b.Exponential.MaxRetries != nil {
			def.MaxRetries = int64(*b.Exponential.MaxRetries)
		}
		def.Multiplier = float32(b.Exponential.Multiplier)
		def.RandomizationFactor = float32(b.Exponential.RandomizationFactor)
	} else {
		return def, errors.New("constant or exponential must be configured")
	}

	return def, nil
}
