/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package function_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/function"
)

func TestContext(t *testing.T) {
	ctx := context.Background()
	empty := function.FromContext(ctx)
	assert.Equal(t, function.Function{}, empty)
	fn := function.Function{
		Namespace: "test.v1",
		Operation: "testing",
	}
	fctx := function.ToContext(ctx, fn)
	actual := function.FromContext(fctx)
	assert.Equal(t, fn, actual)
}
