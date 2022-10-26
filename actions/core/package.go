package core

import (
	"context"
	"net/http"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/runtime"
)

var All = []actions.NamedLoader{
	Assign,
	Authorize,
	CallPipeline,
	CallProvider,
	Decode,
	Filter,
	HTTP,
	HTTPResponse,
	Invoke,
	JMESPath,
	JQ,
	Log,
	ReCaptcha,
	Route,
}

type Processor interface {
	LoadPipeline(pl *runtime.Pipeline) (runtime.Runnable, error)
	Pipeline(ctx context.Context, name string, data actions.Data) (interface{}, error)
	Provider(ctx context.Context, namespace, service, function string, data actions.Data) (interface{}, error)
	Event(ctx context.Context, name string, data actions.Data) (interface{}, error)
}

type HTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}
