/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package engine

import (
	"context"
	"fmt"
	"testing"

	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/stretchr/testify/assert"
	"go.uber.org/zap/zapcore"
)

func TestInvoke(t *testing.T) {
	ctx := context.Background()
	var input any = map[string]interface{}{
		"name": "World",
	}
	handler := handler.Handler{Interface: "Greeter", Operation: "SayHello"}
	info := Info{
		Mode:          ModeInvoke,
		LogLevel:      zapcore.ErrorLevel,
		BusFile:       "test-data/greeter.yaml",
		ResourcesFile: "",
		EntityID:      "nothing",
		DeveloperMode: false,
	}
	engine, err := Start(ctx, &info)
	assert.NoError(t, err)
	response, err := engine.Invoke(handler, input)
	assert.Error(t, err)
	assert.Nil(t, response)
	response, err = engine.InvokeUnsafe(handler, input)
	assert.NoError(t, err)
	assert.Equal(t, "Hello, World", response)
	fmt.Println(response)
}
