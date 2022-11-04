package httpresponse

import (
	"context"
	"net/http"
)

type Response struct {
	Status int
	Header http.Header
}

func New() *Response {
	return &Response{
		Status: http.StatusOK,
		Header: http.Header{},
	}
}

type responseKey struct{}

// NewContext creates a new context with incoming `resp` attached.
func NewContext(ctx context.Context, resp *Response) context.Context {
	return context.WithValue(ctx, responseKey{}, resp)
}

func FromContext(ctx context.Context) *Response {
	iface := ctx.Value(responseKey{})
	response, _ := iface.(*Response)
	return response
}
