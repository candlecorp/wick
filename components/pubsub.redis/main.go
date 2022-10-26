package main

import (
	dapr "github.com/dapr-sandbox/components-go-sdk"
	"github.com/dapr/components-contrib/pubsub/redis"
	"github.com/dapr/kit/logger"
)

var log = logger.NewLogger("redis-pubsub-pluggable")

func main() {
	dapr.MustRun(dapr.UsePubSub(redis.NewRedisStreams(log)))
}
