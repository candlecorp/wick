package main

import (
	dapr "github.com/dapr-sandbox/components-go-sdk"
	im "github.com/dapr/components-contrib/state/in-memory"
	"github.com/dapr/kit/logger"
)

var log = logger.NewLogger("in-memory-pluggable")

func main() {
	dapr.MustRun(dapr.UseStateStore(im.NewInMemoryStateStore(log)))
}
