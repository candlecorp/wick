package runtime

import (
	"encoding/json"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"gopkg.in/yaml.v3"

	"github.com/go-logr/logr"
	rootConfig "github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/errorz"
)

type Configuration struct {
	Application   *Application               `json:"application" yaml:"application"`
	Import        []string                   `json:"import" yaml:"import"`
	Transports    map[string]Component       `json:"transports" yaml:"transports"`
	Tracing       *Component                 `json:"tracing" yaml:"tracing"`
	Specs         []Component                `json:"specs" yaml:"specs"`
	Filters       map[string][]Component     `json:"filters" yaml:"filters"`
	Codecs        map[string]Component       `json:"codecs" yaml:"codecs"`
	Resources     map[string]Component       `json:"resources" yaml:"resources"`
	Migrate       map[string]Component       `json:"migrate" yaml:"migrate"`
	Compute       []Component                `json:"compute" yaml:"compute"`
	Resiliency    Resiliency                 `json:"resiliency" yaml:"resiliency"`
	Services      Services                   `json:"services" yaml:"services"`
	Providers     Services                   `json:"providers" yaml:"providers"`
	Events        FunctionPipelines          `json:"events" yaml:"events"`
	Pipelines     FunctionPipelines          `json:"pipelines" yaml:"pipelines"`
	Subscriptions []Subscription             `json:"subscriptions" yaml:"subscriptions"`
	Errors        map[string]errorz.Template `json:"errors" yaml:"errors"`
}

type Application struct {
	ID          string `mapstructure:"id"`
	Version     string `mapstructure:"version"`
	Environment string `mapstructure:"environment"`
}

type Subscription struct {
	Resource  string            `mapstructure:"resource"`
	Topic     string            `mapstructure:"topic"`
	Metadata  map[string]string `mapstructure:"metadata"`
	Codec     string            `mapstructure:"codec"`
	CodecArgs []interface{}     `mapstructure:"codecArgs"`
	Function  string            `mapstructure:"function"`
}

type Component struct {
	Uses string      `json:"uses" yaml:"uses"`
	With interface{} `json:"with" yaml:"with"`
}

type Resiliency struct {
	Timeouts        map[string]Duration         `json:"timeouts" yaml:"timeouts"`
	Retries         map[string]ConfigProperties `json:"retries" yaml:"retries"`
	CircuitBreakers map[string]ConfigProperties `json:"circuitBreakers" yaml:"circuitBreakers"`
}

type ConfigProperties map[string]interface{}

type Services map[string]FunctionPipelines
type FunctionPipelines map[string]Pipeline

type Pipeline struct {
	Name  string `json:"name" yaml:"name"`
	Call  string `json:"call,omitempty" yaml:"call,omitempty" mapstructure:"call"`
	Steps []Step `json:"steps,omitempty" yaml:"steps,omitempty"`
}

type Step struct {
	Name           string      `json:"name" yaml:"name" mapstructure:"name"`
	Call           string      `json:"call,omitempty" yaml:"call,omitempty" mapstructure:"call"`
	Uses           string      `json:"uses,omitempty" yaml:"uses,omitempty" mapstructure:"uses"`
	With           interface{} `json:"with,omitempty" yaml:"with,omitempty" mapstructure:"with"`
	Returns        string      `json:"returns,omitempty" yaml:"returns,omitempty" mapstructure:"returns"`
	Timeout        string      `json:"timeout,omitempty" yaml:"timeout,omitempty" mapstructure:"timeout"`
	Retry          string      `json:"retry,omitempty" yaml:"retry,omitempty" mapstructure:"retry"`
	CircuitBreaker string      `json:"circuitBreaker,omitempty" yaml:"circuitBreaker,omitempty" mapstructure:"circuitBreaker"`
	OnError        *Pipeline   `json:"onError,omitempty" yaml:"onError,omitempty" mapstructure:"onError"`
}

func DefaultConfiguration() Configuration {
	return Configuration{
		// Specs: []Component{
		// 	{
		// 		Uses: "apex",
		// 		With: map[string]interface{}{
		// 			"filename": "spec.apexlang",
		// 		},
		// 	},
		// },
	}
}

func LoadYAML(in io.Reader) (*Configuration, error) {
	data, err := io.ReadAll(in)
	if err != nil {
		return nil, err
	}

	// Replaces ${env:var} or $env:var in the string according to the values
	// of the current environment variables.
	configString := os.Expand(string(data), func(key string) string {
		var value string
		if strings.HasPrefix(key, "env:") {
			value = os.Getenv(key[4:])
		}
		if value == "" {
			return "$" + key
		}
		return value
	})

	r := strings.NewReader(configString)
	c := DefaultConfiguration()
	if err := yaml.NewDecoder(r).Decode(&c); err != nil {
		return nil, err
	}
	return &c, nil
}

type FilenameConfig struct {
	Filename string `mapstructure:"filename"`
}

func NormalizeBaseDir(dir string, with interface{}) interface{} {
	c := FilenameConfig{
		Filename: "spec.apexlang",
	}

	if err := rootConfig.Decode(with, &c); err == nil {
		c.Filename = filepath.Join(dir, c.Filename)
		with = c
	}
	return with
}

func Combine(config *Configuration, dir string, log logr.Logger, configs ...*Configuration) {
	for _, c := range configs {
		// Compute
		for _, c := range c.Compute {
			c.With = NormalizeBaseDir(dir, c.With)
			config.Compute = append(config.Compute, c)
		}

		// Specs
		for _, spec := range c.Specs {
			spec.With = NormalizeBaseDir(dir, spec.With)
			config.Specs = append(config.Specs, spec)
		}

		// Resources
		if len(c.Resources) > 0 && config.Resources == nil {
			config.Resources = make(map[string]Component)
		}
		for k, v := range c.Resources {
			if _, exists := config.Resources[k]; !exists {
				config.Resources[k] = v
			}
		}

		// Filters
		if len(c.Filters) > 0 && config.Filters == nil {
			config.Filters = make(map[string][]Component)
		}
		for k, v := range c.Filters {
			if _, exists := config.Filters[k]; !exists {
				config.Filters[k] = v
			}
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
			config.Resiliency.Retries = make(map[string]ConfigProperties)
		}
		for k, v := range c.Resiliency.Retries {
			if _, exists := config.Resiliency.Retries[k]; !exists {
				config.Resiliency.Retries[k] = v
			}
		}

		if len(c.Resiliency.CircuitBreakers) > 0 && config.Resiliency.CircuitBreakers == nil {
			config.Resiliency.CircuitBreakers = make(map[string]ConfigProperties)
		}
		for k, v := range c.Resiliency.CircuitBreakers {
			if _, exists := config.Resiliency.CircuitBreakers[k]; !exists {
				config.Resiliency.CircuitBreakers[k] = v
			}
		}

		// Services
		if len(c.Services) > 0 && config.Services == nil {
			config.Services = make(Services)
		}
		for k, v := range c.Services {
			existing, exists := config.Services[k]
			if !exists {
				existing = make(FunctionPipelines)
				config.Services[k] = existing
			}
			for k, v := range v {
				if _, exists := existing[k]; !exists {
					existing[k] = v
				}
			}
		}

		// Providers
		if len(c.Providers) > 0 && config.Providers == nil {
			config.Providers = make(Services)
		}
		for k, v := range c.Providers {
			existing, exists := config.Providers[k]
			if !exists {
				existing = make(FunctionPipelines)
				config.Providers[k] = existing
			}
			for k, v := range v {
				if _, exists := existing[k]; !exists {
					existing[k] = v
				}
			}
		}

		// Events
		if len(c.Events) > 0 && config.Events == nil {
			config.Events = make(FunctionPipelines)
		}
		for k, v := range c.Events {
			if _, exists := config.Events[k]; !exists {
				config.Events[k] = v
			}
		}

		// Pipelines
		if len(c.Pipelines) > 0 && config.Pipelines == nil {
			config.Pipelines = make(FunctionPipelines)
		}
		for k, v := range c.Pipelines {
			if _, exists := config.Pipelines[k]; !exists {
				config.Pipelines[k] = v
			}
		}

		// Errors
		if len(c.Errors) > 0 && config.Errors == nil {
			config.Errors = make(map[string]errorz.Template)
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

	return f.DecodeString(str)
}

func (f *FilePath) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}

	return f.DecodeString(str)
}

func (f *FilePath) DecodeString(value string) error {
	*f = FilePath(filepath.Join(configBaseDir, value))

	return nil
}

type Duration time.Duration

func (d *Duration) UnmarshalJSON(data []byte) error {
	var str string
	if err := json.Unmarshal(data, &str); err != nil {
		return err
	}

	return d.DecodeString(str)
}

func (d *Duration) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	if err := unmarshal(&str); err != nil {
		return err
	}

	return d.DecodeString(str)
}

func (d *Duration) DecodeString(str string) error {
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
