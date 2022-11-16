/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package errorz

import (
	"github.com/nanobus/nanobus/pkg/expr"
)

type Resolver func(err error) *Error

type Template struct {
	Type    string             `json:"type,omitempty" yaml:"type,omitempty" mapstructure:"type"`
	Code    ErrCode            `json:"code" yaml:"code" mapstructure:"code"`
	Status  int                `json:"status,omitempty" yaml:"status,omitempty" mapstructure:"status"`
	Title   *expr.Text         `json:"title,omitempty" yaml:"title,omitempty" mapstructure:"title"`
	Message *expr.Text         `json:"message,omitempty" yaml:"message,omitempty" mapstructure:"message"`
	Path    string             `json:"path,omitempty" yaml:"path,omitempty" mapstructure:"path"`
	Help    *expr.Text         `json:"help,omitempty" yaml:"help,omitempty" mapstructure:"help"`
	Locales map[string]Strings `json:"locales,omitempty" yaml:"locales,omitempty" mapstructure:"locales"`
}

type Strings struct {
	Title   *expr.Text `json:"title,omitempty" yaml:"title,omitempty" mapstructure:"title"`
	Message *expr.Text `json:"message,omitempty" yaml:"message,omitempty" mapstructure:"message"`
}
