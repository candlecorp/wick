package dapr

import (
	"github.com/nanobus/nanobus/actions"
)

var All = []actions.NamedLoader{
	Publish,
	DeleteState,
	GetState,
	SetState,
	InvokeBinding,
}
