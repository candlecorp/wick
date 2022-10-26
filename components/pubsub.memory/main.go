package main

import (
	dapr "github.com/dapr-sandbox/components-go-sdk"
	memory "github.com/dapr/components-contrib/pubsub/in-memory"
	"github.com/dapr/kit/logger"
)

var log = logger.NewLogger("memory-pubsub-pluggable")

func main() {
	dapr.MustRun(dapr.UsePubSub(memory.New(log)))
}
