package main

import (
	dapr "github.com/dapr-sandbox/components-go-sdk"
	redis "github.com/dapr/components-contrib/state/redis"
	"github.com/dapr/kit/logger"
)

var log = logger.NewLogger("redis-pluggable")

func main() {
	dapr.MustRun(dapr.UseStateStore(redis.NewRedisStateStore(log)))
}
