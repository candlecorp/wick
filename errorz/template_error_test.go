package errorz_test

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/errorz"
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
