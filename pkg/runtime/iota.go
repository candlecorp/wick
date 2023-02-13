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
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/oci"
)

const (
	iotaConfigFile = "iota.yaml"
	busConfigFile  = "bus.yaml"
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

var replaceRefChars = []rune{'/', ':', '.'}

func LoadIotaConfig(log logr.Logger, url string, baseUrl string) (IotaConfig, error) {
	h := IotaConfig{}
	log.Info("Importing configuration from", "location", url)

	var busFile string
	var configBaseUrl string
	ref := url

	if oci.IsImageReference(ref) {
		slug := ref
		for _, r := range replaceRefChars {
			slug = strings.ReplaceAll(slug, string(r), "_")
		}
		configBaseUrl = filepath.Join(baseUrl, "iotas", slug)
		stat, err := os.Stat(configBaseUrl)
		if os.IsNotExist(err) {
			if err := os.MkdirAll(configBaseUrl, 0755); err != nil {
				return h, err
			}

			log.Info("Pulling OCI image", "ref", ref)
			busFile, err = oci.Pull(ref, configBaseUrl)
			if err != nil {
				fmt.Printf("Error pulling image: %s\n", err)
				return h, err
			}
			busFile = filepath.Join(configBaseUrl, busFile)
		} else if err != nil {
			return h, err
		} else if !stat.IsDir() {
			return h, fmt.Errorf("%s exists but is not a directory", configBaseUrl)
		} else {
			busFile = findBusFile(configBaseUrl)
		}
	} else {
		path := filepath.Join(baseUrl, ref)
		info, err := os.Stat(path)
		if err != nil {
			return h, err
		}

		configBaseUrl = path
		if info.IsDir() {
			busFile = findBusFile(path)
			log.Info("Using configuration at", "location", busFile)
		} else {
			busFile = path
			configBaseUrl = filepath.Dir(path)
		}
	}

	if err := config.LoadYamlFile(busFile, &h, false); err != nil {
		return h, err
	}

	h.BaseURL = &configBaseUrl
	return h, nil
}

func findBusFile(path string) string {
	// Use iota.yaml, if it exists. Fallback on bus.yaml.
	busFile := filepath.Join(path, iotaConfigFile)
	stat, err := os.Stat(busFile)
	if os.IsNotExist(err) || (stat != nil && stat.IsDir()) {
		busFile = filepath.Join(path, busConfigFile)
	}

	return busFile
}
