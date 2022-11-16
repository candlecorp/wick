/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package expr

import (
	"encoding/json"
	"reflect"
	"strings"
	"text/template"
)

type Text struct {
	tmpl *template.Template
}

func (t *Text) UnmarshalJSON(data []byte) error {
	var str string
	if err := json.Unmarshal(data, &str); err != nil {
		return err
	}
	return t.Parse(str)
}

func (t *Text) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}
	return t.Parse(str)
}

func (t *Text) Parse(str string) error {
	tmpl, err := template.New("text").Funcs(template.FuncMap{
		"pick": pick,
	}).Parse(str)
	if err != nil {
		return err
	}

	*t = Text{
		tmpl: tmpl,
	}

	return nil
}

func (t *Text) Eval(data interface{}) (string, error) {
	var out strings.Builder
	if err := t.tmpl.Execute(&out, data); err != nil {
		return "", err
	}
	return out.String(), nil
}

func pick(args ...interface{}) interface{} {
	for _, arg := range args {
		if !isNil(arg) {
			return arg
		}
	}
	return ""
}

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
}
