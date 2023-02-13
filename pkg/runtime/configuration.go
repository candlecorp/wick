/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package runtime

import (
	"encoding/json"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/go-logr/logr"

	rootConfig "github.com/nanobus/nanobus/pkg/config"
)

type Application struct {
	ID      string
	Version string
}

func LoadResourcesYAML(in io.Reader) (*ResourcesConfig, error) {
	data, err := io.ReadAll(in)
	if err != nil {
		return nil, err
	}

	c := DefaultResourcesConfig()

	if err := rootConfig.LoadYAML(string(data), &c, true); err != nil {
		return nil, err
	}
	return &c, nil
}

func LoadBusYAML(baseDir string, in io.Reader) (*BusConfig, error) {
	data, err := io.ReadAll(in)
	if err != nil {
		return nil, err
	}

	c := DefaultBusConfig()

	if err := rootConfig.LoadYAML(string(data), &c, true); err != nil {
		return nil, err
	}
	c.BaseURL = &baseDir

	return &c, nil
}

type FilenameConfig struct {
	Filename string `mapstructure:"filename"`
}

func NormalizeBaseDir(dir string, with interface{}) interface{} {
	c := FilenameConfig{
		Filename: "apex.axdl",
	}

	if err := rootConfig.Decode(with, &c); err == nil {
		c.Filename = filepath.Join(dir, c.Filename)
		with = c
	}
	return with
}

func Combine(config *BusConfig, dir string, log logr.Logger, configs ...*BusConfig) {
	for _, c := range configs {
		// // Compute
		// for _, c := range c.Compute {
		// 	c.With = NormalizeBaseDir(dir, c.With)
		// 	config.Compute = append(config.Compute, c)
		// }

		// // Specs
		// for _, spec := range c.Specs {
		// 	spec.With = NormalizeBaseDir(dir, spec.With)
		// 	config.Specs = append(config.Specs, spec)
		// }

		// Resources
		// if len(c.Resources) > 0 && config.Resources == nil {
		// 	config.Resources = make(map[string]Component)
		// }
		// for k, v := range c.Resources {
		// 	if _, exists := config.Resources[k]; !exists {
		// 		config.Resources[k] = v
		// 	}
		// }
		if len(c.Resources) > 0 {
			config.Resources = append(config.Resources, c.Resources...)
		}

		// Filters
		// if len(c.Filters) > 0 && config.Filters == nil {
		// 	config.Filters = make(map[string][]Component)
		// }
		// for k, v := range c.Filters {
		// 	if _, exists := config.Filters[k]; !exists {
		// 		config.Filters[k] = v
		// 	}
		// }
		if len(c.Filters) > 0 {
			config.Filters = append(config.Filters, c.Filters...)
		}

		// Codecs
		if len(c.Codecs) > 0 && config.Codecs == nil {
			config.Codecs = make(map[string]Component)
		}
		for k, v := range c.Codecs {
			if _, exists := config.Codecs[k]; !exists {
				config.Codecs[k] = v
			}
		}

		// Resiliency
		if len(c.Resiliency.Timeouts) > 0 && config.Resiliency.Timeouts == nil {
			config.Resiliency.Timeouts = make(map[string]Duration)
		}
		for k, v := range c.Resiliency.Timeouts {
			if _, exists := config.Resiliency.Timeouts[k]; !exists {
				config.Resiliency.Timeouts[k] = v
			}
		}

		if len(c.Resiliency.Retries) > 0 && config.Resiliency.Retries == nil {
			config.Resiliency.Retries = make(map[string]Backoff)
		}
		for k, v := range c.Resiliency.Retries {
			if _, exists := config.Resiliency.Retries[k]; !exists {
				config.Resiliency.Retries[k] = v
			}
		}

		if len(c.Resiliency.CircuitBreakers) > 0 && config.Resiliency.CircuitBreakers == nil {
			config.Resiliency.CircuitBreakers = make(map[string]CircuitBreaker)
		}
		for k, v := range c.Resiliency.CircuitBreakers {
			if _, exists := config.Resiliency.CircuitBreakers[k]; !exists {
				config.Resiliency.CircuitBreakers[k] = v
			}
		}

		// Services
		if len(c.Interfaces) > 0 && config.Interfaces == nil {
			config.Interfaces = make(Interfaces)
		}
		for k, v := range c.Interfaces {
			existing, exists := config.Interfaces[k]
			if !exists {
				existing = make(Operations)
				config.Interfaces[k] = existing
			}
			for k, v := range v {
				if _, exists := existing[k]; !exists {
					existing[k] = v
				}
			}
		}

		// Providers
		if len(c.Providers) > 0 && config.Providers == nil {
			config.Providers = make(Interfaces)
		}
		for k, v := range c.Providers {
			existing, exists := config.Providers[k]
			if !exists {
				existing = make(Operations)
				config.Providers[k] = existing
			}
			for k, v := range v {
				if _, exists := existing[k]; !exists {
					existing[k] = v
				}
			}
		}

		// // Events
		// if len(c.Events) > 0 && config.Events == nil {
		// 	config.Events = make(FunctionPipelines)
		// }
		// for k, v := range c.Events {
		// 	if _, exists := config.Events[k]; !exists {
		// 		config.Events[k] = v
		// 	}
		// }

		// // Pipelines
		// if len(c.Pipelines) > 0 && config.Pipelines == nil {
		// 	config.Pipelines = make(FunctionPipelines)
		// }
		// for k, v := range c.Pipelines {
		// 	if _, exists := config.Pipelines[k]; !exists {
		// 		config.Pipelines[k] = v
		// 	}
		// }

		// Errors
		if len(c.Errors) > 0 && config.Errors == nil {
			config.Errors = make(map[string]ErrorTemplate)
		}
		for k, v := range c.Errors {
			if _, exists := config.Errors[k]; !exists {
				config.Errors[k] = v
			}
		}
	}
}

var configBaseDir string

func SetConfigBaseDir(path string) {
	configBaseDir = path
}

type FilePath string

func (f *FilePath) Relative() string {
	if f == nil {
		return ""
	}

	p := string(*f)
	path, err := os.Getwd()
	if err != nil {
		return p
	}

	rel, err := filepath.Rel(path, p)
	if err != nil {
		return p
	}

	return rel
}

func (f *FilePath) UnmarshalJSON(data []byte) error {
	var str string
	if err := json.Unmarshal(data, &str); err != nil {
		return err
	}

	return f.FromString(str)
}

func (f *FilePath) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}

	return f.FromString(str)
}

func (f *FilePath) FromString(value string) error {
	*f = FilePath(filepath.Join(configBaseDir, value))

	return nil
}

type Duration time.Duration

func (d *Duration) UnmarshalJSON(data []byte) error {
	var str string
	if err := json.Unmarshal(data, &str); err != nil {
		return err
	}

	return d.FromString(str)
}

func (d *Duration) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}

	return d.FromString(str)
}

func (d *Duration) FromString(str string) error {
	millis, err := strconv.ParseUint(str, 10, 32)
	if err == nil {
		*d = Duration(millis) * Duration(time.Millisecond)
		return nil
	}

	dur, err := time.ParseDuration(str)
	if err != nil {
		return err
	}

	*d = Duration(dur)

	return nil
}

func (t *ErrCode) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}
	return t.FromString(str)
}
