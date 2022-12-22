/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package core

import (
	"context"
	"net/http"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/runtime"
)

var All = []actions.NamedLoader{
	Assign,
	Authorize,
	CallInterface,
	CallProvider,
	Decode,
	Expr,
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
	Interface(ctx context.Context, name, function string, data actions.Data) (interface{}, bool, error)
	Provider(ctx context.Context, name, function string, data actions.Data) (interface{}, bool, error)
}

type HTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}
