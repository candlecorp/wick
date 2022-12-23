package runtime

import (
	"time"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resiliency/retry"
)

// DecodeConfig decodes a Go struct into a `BackOffConfig`.
func DecodeConfig(input interface{}) (retry.Config, error) {
	def := retry.DefaultConfig
	c := Backoff{}
	err := config.Decode(input, &c)
	if c.Constant != nil {
		duration, err := config.DecodeDuration(c.Constant.Duration)
		if err != nil {
			return def, nil
		}
		def.Duration = duration
		if c.Constant.MaxRetries != nil {
			def.MaxRetries = int64(*c.Constant.MaxRetries)
		}
	} else if c.Exponential != nil {
		def.InitialInterval = time.Duration(c.Exponential.InitialInterval)
		def.MaxInterval = time.Duration(c.Exponential.MaxInterval)
		def.MaxElapsedTime = time.Duration(c.Exponential.MaxElapsedTime)

		if c.Exponential.MaxRetries != nil {
			def.MaxRetries = int64(*c.Exponential.MaxRetries)
		}
		def.Multiplier = float32(c.Exponential.Multiplier)
		def.RandomizationFactor = float32(c.Exponential.RandomizationFactor)
	}

	return def, err
}
