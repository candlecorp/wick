/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package transport_test

import (
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/transport"
)

func TestRegistry(t *testing.T) {
	r := transport.Registry{}

	loader := func(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (transport.Transport, error) {
		return nil, nil
	}
	namedLoader := func() (string, transport.Loader) {
		return "test", loader
	}

	r.Register(namedLoader)

	assert.Equal(t, fmt.Sprintf("%v", transport.Loader(loader)), fmt.Sprintf("%p", r["test"]))
}
