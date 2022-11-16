/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package errorz_test

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/pkg/errorz"
)

func TestParse(t *testing.T) {
	message := `customer_not_found
[store] statestore
[key] 12345`

	te := errorz.ParseTemplateError(message)
	assert.Equal(t, "customer_not_found", te.Template)
	assert.Equal(t, errorz.Metadata{
		"store": "statestore",
		"key":   "12345",
	}, te.Metadata)
}
