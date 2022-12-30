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
		if b.Constant.Duration != 0 {
			def.Duration = time.Duration(b.Constant.Duration)
		}
		if b.Constant.MaxRetries != nil {
			def.MaxRetries = int64(*b.Constant.MaxRetries)
		}
	} else if b.Exponential != nil {
		def.Policy = retry.PolicyExponential
		if b.Exponential.InitialInterval != 0 {
			def.InitialInterval = time.Duration(b.Exponential.InitialInterval)
		}
		if b.Exponential.MaxInterval != 0 {
			def.MaxInterval = time.Duration(b.Exponential.MaxInterval)
		}
		if b.Exponential.MaxElapsedTime != 0 {
			def.MaxElapsedTime = time.Duration(b.Exponential.MaxElapsedTime)
		}
		if b.Exponential.MaxRetries != nil {
			def.MaxRetries = int64(*b.Exponential.MaxRetries)
		}
		if b.Exponential.Multiplier != 0 {
			def.Multiplier = float32(b.Exponential.Multiplier)
		}
		if b.Exponential.RandomizationFactor != 0 {
			def.RandomizationFactor = float32(b.Exponential.RandomizationFactor)
		}
	} else {
		return def, errors.New("constant or exponential must be configured")
	}

	return def, nil
}
