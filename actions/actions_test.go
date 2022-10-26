package actions_test

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/nanobus/nanobus/actions"
)

func TestClone(t *testing.T) {
	data := actions.Data{
		"one": 1234,
	}
	clone := data.Clone()
	assert.Equal(t, data, clone)
	assert.NotSame(t, data, clone)
}

func TestStop(t *testing.T) {
	assert.Equal(t, actions.ErrStop, actions.Stop())
}
