package config

import (
	"io"
	"os"
	"strings"

	"gopkg.in/yaml.v3"
)

func LoadYAML(src string, obj interface{}) error {
	configString := os.Expand(src, envReplace)
	r := strings.NewReader(configString)
	decoder := yaml.NewDecoder(r)
	decoder.KnownFields(true)
	err := decoder.Decode(obj)
	if err != nil {
		return err
	}
	return AssertInitialized(obj)
}

func LoadYamlFile(filename string, obj interface{}) error {
	f, err := os.OpenFile(filename, os.O_RDONLY, 0644)
	if err != nil {
		return err
	}
	defer f.Close()

	data, err := io.ReadAll(f)
	if err != nil {
		return err
	}

	return LoadYAML(string(data), obj)
}

// Replaces ${env:var} or $env:var in the string according to the values
// of the current environment variables.
func envReplace(key string) string {
	var value string
	if strings.HasPrefix(key, "env:") {
		value = os.Getenv(key[4:])
	}
	if value == "" {
		return "$" + key
	}
	return value
}
