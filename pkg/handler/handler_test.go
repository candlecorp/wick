/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package handler_test

import (
	"context"
	"testing"

	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/stretchr/testify/assert"
)

func TestContext(t *testing.T) {
	ctx := context.Background()
	empty := handler.FromContext(ctx)
	assert.Equal(t, handler.Handler{}, empty)
	fn := handler.Handler{
		Interface: "test.v1",
		Operation: "testing",
	}
	fctx := handler.ToContext(ctx, fn)
	actual := handler.FromContext(fctx)
	assert.Equal(t, fn, actual)
}
