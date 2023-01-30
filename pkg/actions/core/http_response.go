/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core

import (
	"context"
	"fmt"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/transport/httpresponse"
)

func HTTPResponseLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := HTTPResponseConfig{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	return HTTPResponseAction(&c), nil
}

func HTTPResponseAction(
	config *HTTPResponseConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		resp := httpresponse.FromContext(ctx)
		if resp == nil {
			return nil, nil
		}

		if config.Status != nil {
			resp.Status = int(*config.Status)
		}

		for _, h := range config.Headers {
			val, err := h.Value.Eval(data)
			if err != nil {
				return nil, err
			}
			resp.Header.Add(h.Name, fmt.Sprintf("%v", val))
		}

		return nil, nil
	}
}
