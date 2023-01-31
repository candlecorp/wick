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
	"fmt"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/go-redis/redis/v8"
)

func TestRemoveGet(t *testing.T) {
	ctx := context.Background()

	client := redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "",
		DB:       0,
	})

	randomKey := fmt.Sprint(rand.Int())
	randomNumber := fmt.Sprint(rand.Int())

	setConfig := SetConfig{
		Key:   randomKey,
		Value: randomNumber,
	}

	result, err := SetAction(&setConfig, client)(ctx, nil)

	assert.NoError(t, err)
	assert.Equal(t, "OK", result)

	config := GetConfig{
		Key: randomKey,
	}

	action := GetAction(&config, client)
	result, err = action(ctx, nil)
	assert.NoError(t, err)
	assert.Equal(t, randomNumber, result)
}
