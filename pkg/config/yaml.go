package config

import (
	"errors"
	"fmt"
	"io"
	"os"
	"regexp"
	"strings"

	"gopkg.in/yaml.v3"
)

var reYamlError = regexp.MustCompile(`line (\d+): field ([a-zA-Z0-9_-]+) not found .*$`)

func LoadYAML(src string, obj interface{}, knownFields bool) error {
	configString := os.Expand(src, envReplace)
	r := strings.NewReader(configString)
	decoder := yaml.NewDecoder(r)
	decoder.KnownFields(knownFields)
	if err := decoder.Decode(obj); err != nil {
		// Test if the error is a yaml TypeError
		if typeError, ok := err.(*yaml.TypeError); ok {
			errs := []string{}
			// Then extract the valuable data from the errors and make them more useful.
			for _, invalidField := range typeError.Errors {
				found := reYamlError.FindStringSubmatch(invalidField)
				if len(found) == 3 {
					line := found[1]
					field := found[2]
					errs = append(errs, fmt.Sprintf("line %s: field '%s' not expected", line, field))
				}
			}
			return errors.New(strings.Join(errs, ", "))
		}
		return err
	}
	return validate.Struct(obj)
}

func LoadYamlFile(filename string, obj interface{}, knownFields bool) error {
	f, err := os.OpenFile(filename, os.O_RDONLY, 0644)
	if err != nil {
		return err
	}
	defer f.Close()

	data, err := io.ReadAll(f)
	if err != nil {
		return err
	}

	return LoadYAML(string(data), obj, knownFields)
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
