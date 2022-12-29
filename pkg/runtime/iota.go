/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package runtime

import (
	"context"
	"errors"
	"os"
	"path/filepath"

	"github.com/go-logr/logr"
	"github.com/nanobus/nanobus/pkg/config"

	"github.com/nanobus/nanobus/pkg/oci"
)

type iotaKey struct{}

func FromContext(ctx context.Context) IotaConfig {
	v := ctx.Value(iotaKey{})
	if v == nil {
		return IotaConfig{}
	}
	c, _ := v.(IotaConfig)

	return c
}

func ToContext(ctx context.Context, function IotaConfig) context.Context {
	return context.WithValue(ctx, iotaKey{}, function)
}

func LoadIotaConfig(log logr.Logger, url string, baseUrl string) (IotaConfig, error) {
	h := IotaConfig{}
	log.Info("Importing configuration from", "location", url)

	var busFile string
	ref := url

	if oci.IsImageReference(ref) {
		return h, errors.New("references not currently supported")
	}

	path := filepath.Join(baseUrl, ref)
	info, err := os.Stat(path)
	if err != nil {
		return h, err
	}

	configBaseUrl := path
	if info.IsDir() {
		busFile = filepath.Join(path, "iota.yaml")
		log.Info("Using configuration at", "location", busFile)
	} else {
		busFile = path
		configBaseUrl = filepath.Dir(path)
	}

	err = config.LoadYamlFile(busFile, &h)
	if err != nil {
		return h, err
	}

	h.BaseURL = &configBaseUrl
	return h, nil
}
