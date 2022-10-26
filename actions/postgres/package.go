package postgres

import (
	"github.com/nanobus/nanobus/actions"
)

var All = []actions.NamedLoader{
	Load,
	Find,
	FindOne,
	Query,
	Exec,
	ExecMulti,
	Test,
}
