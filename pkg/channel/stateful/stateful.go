/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package stateful

type RawItem struct {
	Namespace  string `json:"namespace,omitempty" msgpack:"namespace,omitempty"`
	Type       string `json:"type,omitempty" msgpack:"type,omitempty"`
	Version    string `json:"version,omitempty" msgpack:"version,omitempty"`
	Data       []byte `json:"data,omitempty" msgpack:"data,omitempty"`
	DataBase64 string `json:"dataBase64,omitempty" msgpack:"dataBase64,omitempty"`
}

type Mutation struct {
	Set    map[string]RawItem `json:"set,omitempty" msgpack:"set,omitempty"`
	Remove []string           `json:"remove,omitempty" msgpack:"remove,omitempty"`
}

type Response struct {
	Mutation Mutation    `json:"mutation,omitempty" msgpack:"mutation,omitempty"`
	Result   interface{} `json:"result,omitempty" msgpack:"result,omitempty"`
}
