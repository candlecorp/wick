/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package handler

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
)

type Handler struct {
	Interface string `json:"interface" msgpack:"interface"`
	Operation string `json:"operation" msgpack:"operation"`
}

type handlerKey struct{}

func FromContext(ctx context.Context) Handler {
	v := ctx.Value(handlerKey{})
	if v == nil {
		return Handler{}
	}
	c, _ := v.(Handler)

	return c
}

func ToContext(ctx context.Context, function Handler) context.Context {
	return context.WithValue(ctx, handlerKey{}, function)
}

func (h *Handler) String() string {
	return h.Interface + "::" + h.Operation
}

func (h *Handler) FromString(handlerName string) error {
	parts := strings.Split(handlerName, "::")
	if len(parts) != 2 {
		return fmt.Errorf("invalid handler format %q", handlerName)
	}
	*h = Handler{
		Interface: parts[0],
		Operation: parts[1],
	}
	return nil
}

// UnmarshalJSON unmashals a quoted json string to the enum value
func (h *Handler) UnmarshalJSON(b []byte) error {
	var str string
	err := json.Unmarshal(b, &str)
	if err != nil {
		return err
	}
	return h.FromString(str)
}

func (h *Handler) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}
	return h.FromString(str)
}
