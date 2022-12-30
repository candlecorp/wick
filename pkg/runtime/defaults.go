package runtime

import (
	"time"
)

// Returns a ResourcesConfig instance with default fields populated

func DefaultResourcesConfig() ResourcesConfig {
	obj := ResourcesConfig{}

	return obj
}

func (h *ResourcesConfig) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias ResourcesConfig
	raw := alias(DefaultResourcesConfig())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = ResourcesConfig(raw)
	return nil
}

// Returns a BusConfig instance with default fields populated

func DefaultBusConfig() BusConfig {
	obj := BusConfig{}

	return obj
}

func (h *BusConfig) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias BusConfig
	raw := alias(DefaultBusConfig())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = BusConfig(raw)
	return nil
}

// Returns a IotaConfig instance with default fields populated

func DefaultIotaConfig() IotaConfig {
	obj := IotaConfig{}

	return obj
}

func (h *IotaConfig) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias IotaConfig
	raw := alias(DefaultIotaConfig())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = IotaConfig(raw)
	return nil
}

// Returns a Package instance with default fields populated

func DefaultPackage() Package {
	obj := Package{}

	return obj
}

func (h *Package) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Package
	raw := alias(DefaultPackage())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Package(raw)
	return nil
}

// Returns a Reference instance with default fields populated

func DefaultReference() Reference {
	obj := Reference{}

	return obj
}

func (h *Reference) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Reference
	raw := alias(DefaultReference())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Reference(raw)
	return nil
}

// Returns a Component instance with default fields populated

func DefaultComponent() Component {
	obj := Component{}

	return obj
}

func (h *Component) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Component
	raw := alias(DefaultComponent())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Component(raw)
	return nil
}

// Returns a Resiliency instance with default fields populated

func DefaultResiliency() Resiliency {
	obj := Resiliency{}

	return obj
}

func (h *Resiliency) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Resiliency
	raw := alias(DefaultResiliency())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Resiliency(raw)
	return nil
}

// Returns a ConstantBackoff instance with default fields populated

func DefaultConstantBackoff() ConstantBackoff {
	obj := ConstantBackoff{}

	return obj
}

func (h *ConstantBackoff) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias ConstantBackoff
	raw := alias(DefaultConstantBackoff())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = ConstantBackoff(raw)
	return nil
}

// Returns a ExponentialBackoff instance with default fields populated

func DefaultExponentialBackoff() ExponentialBackoff {
	obj := ExponentialBackoff{}
	obj.InitialInterval = (func(value string) Duration { d, _ := time.ParseDuration(value); return Duration(d) })("500ms")
	obj.RandomizationFactor = 0.5
	obj.Multiplier = 1.5
	obj.MaxInterval = (func(value string) Duration { d, _ := time.ParseDuration(value); return Duration(d) })("60s")
	obj.MaxElapsedTime = (func(value string) Duration { d, _ := time.ParseDuration(value); return Duration(d) })("15m")

	return obj
}

func (h *ExponentialBackoff) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias ExponentialBackoff
	raw := alias(DefaultExponentialBackoff())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = ExponentialBackoff(raw)
	return nil
}

// Returns a CircuitBreaker instance with default fields populated

func DefaultCircuitBreaker() CircuitBreaker {
	obj := CircuitBreaker{}
	obj.MaxRequests = 1
	obj.Interval = (func(value string) Duration { d, _ := time.ParseDuration(value); return Duration(d) })("0s")
	obj.Timeout = (func(value string) Duration { d, _ := time.ParseDuration(value); return Duration(d) })("60s")

	return obj
}

func (h *CircuitBreaker) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias CircuitBreaker
	raw := alias(DefaultCircuitBreaker())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = CircuitBreaker(raw)
	return nil
}

// Returns a Authorization instance with default fields populated

func DefaultAuthorization() Authorization {
	obj := Authorization{}
	obj.Unauthenticated = false

	return obj
}

func (h *Authorization) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Authorization
	raw := alias(DefaultAuthorization())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Authorization(raw)
	return nil
}

// Returns a Pipeline instance with default fields populated

func DefaultPipeline() Pipeline {
	obj := Pipeline{}

	return obj
}

func (h *Pipeline) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Pipeline
	raw := alias(DefaultPipeline())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Pipeline(raw)
	return nil
}

// Returns a Step instance with default fields populated

func DefaultStep() Step {
	obj := Step{}

	return obj
}

func (h *Step) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Step
	raw := alias(DefaultStep())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Step(raw)
	return nil
}

// Returns a ErrorTemplate instance with default fields populated

func DefaultErrorTemplate() ErrorTemplate {
	obj := ErrorTemplate{}

	return obj
}

func (h *ErrorTemplate) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias ErrorTemplate
	raw := alias(DefaultErrorTemplate())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = ErrorTemplate(raw)
	return nil
}

// Returns a Strings instance with default fields populated

func DefaultStrings() Strings {
	obj := Strings{}

	return obj
}

func (h *Strings) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias Strings
	raw := alias(DefaultStrings())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = Strings(raw)
	return nil
}
