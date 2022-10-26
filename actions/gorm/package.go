package gorm

import (
	"github.com/nanobus/nanobus/actions"
)

var All = []actions.NamedLoader{
	Find,
	FindAll,
}
