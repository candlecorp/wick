/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package core

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/propagation"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/coalesce"
	"github.com/nanobus/nanobus/pkg/codec"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
)

func HTTPLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := HTTPConfig{
		Codec: "json",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var httpClient HTTPClient
	var codecs codec.Codecs
	if err := resolve.Resolve(resolver,
		"client:http", &httpClient,
		"codec:lookup", &codecs); err != nil {
		return nil, err
	}

	codec, ok := codecs[c.Codec]
	if !ok {
		return nil, fmt.Errorf("unknown codec %q", c.Codec)
	}

	return HTTPAction(httpClient, codec, codecs, &c), nil
}

func HTTPAction(
	httpClient HTTPClient,
	codec codec.Codec,
	codecs codec.Codecs,
	config *HTTPConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var err error
		var requestBody io.Reader

		if config.Body != nil {
			requestData, err := config.Body.Eval(data)
			if err != nil {
				return nil, err
			}
			requestData, _ = coalesce.ToMapSI(requestData, true)
			requestBytes, err := json.Marshal(requestData)
			if err != nil {
				return nil, err
			}
			requestBody = bytes.NewReader(requestBytes)
		}

		req, err := http.NewRequestWithContext(
			ctx,
			config.Method,
			config.URL,
			requestBody)
		if err != nil {
			return nil, err
		}

		otel.GetTextMapPropagator().Inject(ctx, propagation.HeaderCarrier(req.Header))

		req.Header.Set("Content-Type", codec.ContentType())
		if config.Headers != nil {
			headers, err := config.Headers.EvalMap(data)
			if err != nil {
				return nil, err
			}
			for name, value := range headers {
				req.Header.Set(name, value)
			}
		}

		resp, err := httpClient.Do(req)
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()

		if resp.StatusCode/100 != 2 {
			return nil, fmt.Errorf("expected 2XX status code; received %d", resp.StatusCode)
		}

		respCodec := codec
		respContentType := resp.Header.Get("Content-Type")
		if respContentType != "" {
			if c, ok := codecs[respContentType]; ok {
				respCodec = c
			}
		}

		responseBytes, err := io.ReadAll(resp.Body)
		if err != nil {
			return nil, err
		}

		var response interface{}
		if len(responseBytes) > 0 {
			if response, _, err = respCodec.Decode(responseBytes, config.CodecArgs...); err != nil {
				return nil, err
			}

			responseMap, ok := response.(map[string]interface{})
			if ok && config.Output != nil {
				response, err = config.Output.Eval(responseMap)
				if err != nil {
					return nil, err
				}

				response = coalesce.ValueIItoSI(response, true)
			}
		}

		return response, err
	}
}
