/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package redis

import (
	"context"
	"testing"

	"github.com/go-redis/redis/v8"
	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/actions/blob"
	"github.com/nanobus/nanobus/pkg/codec"
	json_codec "github.com/nanobus/nanobus/pkg/codec/json"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGet(t *testing.T) {
	ctx := context.Background()

	client := redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "",
		DB:       0,
	})

	jsonCodec := json_codec.NewCodec()
	codecs := codec.Codecs{
		"json": jsonCodec,
	}

	resolver := func(name string, target interface{}) bool {
		switch name {
		case "resource:lookup":
			return resolve.As(client, target)
		case "codec:lookup":
			return resolve.As(codecs, target)
		}
		return false
	}

	expected := map[string]any{
		"foo": "bar",
	}

	data := actions.Data{
		"input": map[string]any{
			"key": "1234",
		},
	}

	a, err := blob.ReadLoader(ctx, map[string]any{
		"resource": "test",
		"key":      "input.key",
		"codec":    "json",
	}, resolver)
	require.NoError(t, err)

	actual, err := a(ctx, data)
	require.NoError(t, err)

	assert.Equal(t, expected, actual)

	// randomKey := fmt.Sprint(rand.Int())
	// randomNumber := fmt.Sprint(rand.Int())

	// expected := map[string]any{
	// 	randomKey: randomNumber,
	// }

	// setConfig := SetConfig{
	// 	Key:   randomKey,
	// 	Value: randomNumber,
	// 	Codec: "bytes",
	// }
	// setCodec := setConfig.Codec

	// result, err := SetAction(&setConfig, nil, client)(ctx, nil)

	// assert.NoError(t, err)
	// assert.Equal(t, "OK", result)

	// config := GetConfig{
	// 	Key:   randomKey,
	// 	Codec: "bytes",
	// }

	// getCodec := config.Codec

	// action := GetAction(&config, nil, client)
	// result, err = action(ctx, nil)
	// assert.NoError(t, err)
	// assert.Equal(t, randomNumber, result)
}
