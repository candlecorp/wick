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

	"github.com/jmespath/go-jmespath"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
)

func JMESPathLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	var c JMESPathConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	jp, err := jmespath.Compile(c.Path)
	if err != nil {
		return nil, fmt.Errorf("error compiling jmespath query: %w", err)
	}

	return JMESPathAction(&c, jp), nil
}

func JMESPathAction(
	config *JMESPathConfig, jp *jmespath.JMESPath) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		var in interface{} = map[string]interface{}(data)
		if config.Data != nil {
			var err error
			in, err = config.Data.Eval(data)
			if err != nil {
				return nil, err
			}
		}

		result, err := safeSearch(in, jp)
		if err != nil {
			return nil, err
		}

		if config.Var != nil {
			data[*config.Var] = result
		}

		return result, nil
	}
}

func safeSearch(v interface{}, j *jmespath.JMESPath) (result interface{}, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("jmespath panic: %v", r)
		}
	}()
	return j.Search(v)
}
