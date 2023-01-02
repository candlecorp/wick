/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package spec

import (
	"github.com/go-playground/locales/en"
	ut "github.com/go-playground/universal-translator"
	"github.com/go-playground/validator/v10"
	en_translations "github.com/go-playground/validator/v10/translations/en"
)

var (
	uni            *ut.UniversalTranslator
	globalValidate *validator.Validate
	translator     ut.Translator
)

func init() {
	en := en.New()
	uni = ut.New(en, en)

	// this is usually know or extracted from http 'Accept-Language' header
	// also see uni.FindTranslator(...)
	var found bool
	translator, found = uni.GetTranslator("en")
	if !found {
		panic("translator not found")
	}

	globalValidate = validator.New()
	if err := en_translations.RegisterDefaultTranslations(globalValidate, translator); err != nil {
		panic(err)
	}
}

// TODO(jsoverson): Eventually delete or use. This is currently unused
// but I'm keeping it to prevent reimplementation.
// var validators = map[string]ValidationLoader{
// 	"url": func(t *TypeRef, f *Field, a *Annotation) (Validation, error) {
// 		return func(v interface{}) ([]ValidationError, error) {
// 			val := validator.New()
// 			value := cast.ToString(v)

// 			if err := val.Var(value, "url"); err != nil {
// 				return []ValidationError{
// 					{
// 						//Fields:  []string{f.Name},
// 						Message: fmt.Sprintf("%q is an invalid URL", f.Name),
// 					},
// 				}, nil
// 			}

// 			return nil, nil
// 		}, nil
// 	},
// 	"email": func(t *TypeRef, f *Field, a *Annotation) (Validation, error) {
// 		return func(v interface{}) ([]ValidationError, error) {
// 			val := validator.New()
// 			value := cast.ToString(v)

// 			if err := val.Var(value, "email"); err != nil {
// 				return []ValidationError{
// 					{
// 						//Fields:  []string{f.Name},
// 						Message: fmt.Sprintf("%q is an invalid E-Mail address", f.Name),
// 					},
// 				}, nil
// 			}

// 			return nil, nil
// 		}, nil
// 	},
// }

var validationRules = map[string]func(a *Annotation) (string, error){
	"url":   func(a *Annotation) (string, error) { return "url", nil },
	"email": func(a *Annotation) (string, error) { return "email", nil },
}
