package errorz

import (
	"fmt"
	"time"
)

type Error struct {
	// Type is a textual type for the error.
	Type string `json:"type,omitempty" yaml:"type,omitempty" msgpack:"type,omitempty"`
	// Code is the numeric error code to return.
	Code ErrCode `json:"code" yaml:"code" msgpack:"code"`
	// Is the transport specific status code for the
	// error type/code.
	Status int `json:"status,omitempty" yaml:"status,omitempty" msgpack:"status,omitempty"`
	// Title is a short message of the error.
	Title string `json:"title,omitempty" yaml:"title,omitempty" msgpack:"title,omitempty"`
	// Message is a descriptive message of the error.
	Message string `json:"message,omitempty" yaml:"message,omitempty" msgpack:"message,omitempty"`
	// Details are user-defined additional details.
	Details interface{} `json:"details,omitempty" yaml:"details,omitempty" msgpack:"details,omitempty"`
	// Metadata is debugging information structured as key-value
	// pairs. Metadata is not exposed to external clients.
	Metadata Metadata `json:"-" yaml:"-" msgpack:"-"`
	// Path indicates the request path that generated the error.
	Path string `json:"path,omitempty" yaml:"path,omitempty" msgpack:"path,omitempty"`
	// Instance is a URI that identifies the specific occurrence of the error.
	Instance string `json:"instance,omitempty" yaml:"instance,omitempty" msgpack:"instance,omitempty"`
	// Help is a URI that hosts additional information about the error.
	Help string `json:"help,omitempty" yaml:"help,omitempty" msgpack:"help,omitempty"`
	// Err is the underlying error if any.
	Err error `json:"-" yaml:"-" msgpack:"-"`
	// Errors encapsulate multiple errors that occurred.
	Errors []*Error `json:"errors,omitempty" yaml:"errors,omitempty" msgpack:"errors,omitempty"`
	// Timestamp is the time in which the error occurred in UTC.
	Timestamp time.Time `json:"timestamp" yaml:"timestamp" msgpack:"timestamp"`
}

type Metadata map[string]interface{}

func From(err error) *Error {
	if err == nil {
		return nil
	}
	if errz, ok := err.(*Error); ok {
		return errz
	}
	return Wrap(err, Unknown)
}

func New(code ErrCode, message ...string) *Error {
	var messageItem string
	if len(message) > 0 {
		messageItem = message[0]
	}
	return &Error{
		Type:      code.String(),
		Code:      code,
		Status:    code.HTTPStatus(),
		Message:   messageItem,
		Timestamp: time.Now().UTC(),
	}
}

func Wrap(err error, code ErrCode, message ...string) *Error {
	var messageItem string
	if len(message) > 0 {
		messageItem = message[0]
	}
	return &Error{
		Type:      code.String(),
		Code:      code,
		Status:    code.HTTPStatus(),
		Message:   messageItem,
		Err:       err,
		Timestamp: time.Now().UTC(),
	}
}

func (e *Error) Error() string {
	return e.Message
}

func (e *Error) Unwrap() error {
	return e.Err
}

type Builder struct {
	err *Error
}

func Build(code ErrCode, err ...error) Builder {
	var e error
	if len(err) > 0 {
		e = err[0]
	}
	return Builder{
		err: Wrap(e, code),
	}
}

func (b Builder) Type(t string) Builder {
	b.err.Type = t
	return b
}

func (b Builder) Title(format string, args ...interface{}) Builder {
	b.err.Title = fmt.Sprintf(format, args...)
	return b
}

func (b Builder) Message(message string) Builder {
	b.err.Message = message
	return b
}

func (b Builder) Messagef(format string, args ...interface{}) Builder {
	b.err.Message = fmt.Sprintf(format, args...)
	return b
}

func (b Builder) Details(details interface{}) Builder {
	b.err.Details = details
	return b
}

func (b Builder) Metadata(metadata Metadata) Builder {
	b.err.Metadata = metadata
	return b
}

func (b Builder) Error(err error) Builder {
	b.err.Err = err
	return b
}

func (b Builder) Instance(instance string) Builder {
	b.err.Instance = instance
	return b
}

func (b Builder) Help(help string) Builder {
	b.err.Help = help
	return b
}

func (b Builder) Multi(errs ...*Error) Builder {
	b.err.Errors = append(b.err.Errors, errs...)
	return b
}

func (b Builder) Err() *Error {
	return b.err
}
