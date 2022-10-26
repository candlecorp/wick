package main

import (
	dapr "github.com/dapr-sandbox/components-go-sdk"
	"github.com/dapr/components-contrib/pubsub/kafka"
	"github.com/dapr/kit/logger"
)

var log = logger.NewLogger("kafka-pluggable")

func main() {
	dapr.MustRun(dapr.UsePubSub(kafka.NewKafka(log)))
}
