/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package redis_test

import (
	"context"
	"testing"

	"github.com/go-redis/redis/v8"
	"github.com/nanobus/nanobus/pkg/actions"
	nanoredis "github.com/nanobus/nanobus/pkg/actions/redis"
	"github.com/nanobus/nanobus/pkg/codec"
	json_codec "github.com/nanobus/nanobus/pkg/codec/json"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRemove(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}
	ctx := context.Background()

	client := redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "",
		DB:       0,
	})

	resources := resource.Resources{
		"test": client,
	}

	jsonCodec := json_codec.NewCodec()
	codecs := codec.Codecs{
		"json": jsonCodec,
	}

	resolver := func(name string, target interface{}) bool {
		switch name {
		case "resource:lookup":
			return resolve.As(resources, target)
		case "codec:lookup":
			return resolve.As(codecs, target)
		}
		return false
	}

	var res int64 = 0

	expected := map[string]any{
		"result": res,
	}

	data := actions.Data{
		"input": map[string]any{
			"key": "foo",
		},
	}

	c, err := nanoredis.RemoveLoader(ctx, map[string]any{
		"resource": "test",
		"key":      "input.key",
		"codec":    "json",
	}, resolver)
	require.NoError(t, err)

	_, err = c(ctx, data)
	require.NoError(t, err)

	actual, err := c(ctx, data)
	require.NoError(t, err)

	map_result := map[string]interface{}{
		"result": actual,
	}

	assert.Equal(t, expected, map_result)
}
