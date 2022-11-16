package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"

	"github.com/spf13/cast"
	"gopkg.in/yaml.v3"

	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/spec"
	"github.com/nanobus/nanobus/pkg/spec/apex"
)

type Config struct {
	Consumer Operation `json:"consumer" yaml:"consumer"`
	Provider Operation `json:"provider" yaml:"provider"`
}

type Operation struct {
	Spec      string `json:"spec" yaml:"spec"`
	Service   string `json:"service" yaml:"service"`
	Operation string `json:"operation" yaml:"operation"`
	Transform string `json:"transform,omitempty" yaml:"transform,omitempty"`
}

func main() {
	args := os.Args
	if len(args) < 2 {
		log.Println("usage: contractcheck <config file>")
		os.Exit(1)
	}

	configBytes, err := os.ReadFile(args[1])
	if err != nil {
		log.Fatal(err)
	}

	config := Config{}
	if err := yaml.Unmarshal(configBytes, &config); err != nil {
		log.Fatal(err)
	}

	consumer, err := loadSpec(config.Consumer.Spec)
	if err != nil {
		log.Fatal(err)
	}

	provider, err := loadSpec(config.Provider.Spec)
	if err != nil {
		log.Fatal(err)
	}

	consumerService, ok := consumer.Service(config.Consumer.Service)
	if !ok {
		log.Fatalf("consumer service %q not found", config.Consumer.Service)
	}
	consumerOperation, ok := consumerService.Operation(config.Consumer.Operation)
	if !ok {
		log.Fatalf("consumer operation %q not found", config.Consumer.Service)
	}

	providerService, ok := provider.Service(config.Provider.Service)
	if !ok {
		log.Fatalf("provider service %q not found", config.Provider.Service)
	}
	providerOperation, ok := providerService.Operation(config.Provider.Operation)
	if !ok {
		log.Fatalf("provider operation %q not found", config.Provider.Service)
	}

	fmt.Println("Parameters check")

	requestPayload := generatePayload(consumerOperation.Parameters, true)
	requestPayload, err = transform(config.Consumer.Transform, requestPayload)
	if err != nil {
		log.Fatal(err)
	}
	printPayload(requestPayload)
	if err := providerOperation.Parameters.Coalesce(requestPayload, true); err != nil {
		log.Fatal(err)
	}

	fmt.Println("Return check")

	if providerOperation.Returns.Kind == spec.KindType &&
		consumerOperation.Returns.Kind == spec.KindType {
		responsePayload := generatePayload(providerOperation.Returns.Type, true)
		responsePayload, err = transform(config.Provider.Transform, responsePayload)
		if err != nil {
			log.Fatal(err)
		}
		printPayload(responsePayload)
		if err := consumerOperation.Returns.Type.Coalesce(responsePayload, true); err != nil {
			log.Fatal(err)
		}
	}
	// TODO else for primitives

	fmt.Println("Contracts are compatible!")
}

func transform(transform string, payload map[string]interface{}) (map[string]interface{}, error) {
	if transform != "" {
		var tx expr.DataExpr
		if err := tx.DecodeString(transform); err != nil {
			return nil, err
		}
		transformed, err := tx.Eval(payload)
		if err != nil {
			return nil, err
		}
		if mt, ok := transformed.(map[string]interface{}); ok {
			return mt, nil
		}
	}
	return payload, nil
}

func loadSpec(filename string) (*spec.Namespace, error) {
	specBytes, err := os.ReadFile(filename)
	if err != nil {
		return nil, err
	}

	return apex.Parse(specBytes)
}

func generatePayload(t *spec.Type, includeOptional bool) map[string]interface{} {
	result := make(map[string]interface{}, len(t.Fields))
	for _, f := range t.Fields {
		if includeOptional || f.Type.Kind != spec.KindOptional {
			sample := getSample(&f.Annotated)
			result[f.Name] = exampleValue(f.Type, sample, includeOptional)
		}
	}
	return result
}

func exampleValue(t *spec.TypeRef, sample string, includeOptional bool) interface{} {
	switch t.Kind {
	case spec.KindOptional:
		if !includeOptional {
			return nil
		}
		return exampleValue(t.OptionalType, sample, includeOptional)
	case spec.KindBool:
		if sample != "" {
			ret, _ := strconv.ParseBool(sample)
			return ret
		}
		return true
	case spec.KindBytes:
		if sample != "" {
			return []byte(sample)
		}
		return []byte("bytes")
	case spec.KindDateTime:
		if sample != "" {
			return sample
		}
		return "2022-05-16T12:00:00Z"
	case spec.KindEnum:
		if sample != "" {
			return sample
		}
		ev := t.Enum.Values[0]
		return ev.StringValue
	case spec.KindF32:
		if sample != "" {
			return cast.ToFloat32(sample)
		}
		return float32(12.34)
	case spec.KindF64:
		if sample != "" {
			return cast.ToFloat64(sample)
		}
		return float32(23.45)
	case spec.KindI8:
		if sample != "" {
			return cast.ToInt8(sample)
		}
		return int8(123)
	case spec.KindI16:
		if sample != "" {
			return cast.ToInt16(sample)
		}
		return int16(234)
	case spec.KindI32:
		if sample != "" {
			return cast.ToInt32(sample)
		}
		return int32(345)
	case spec.KindI64:
		if sample != "" {
			return cast.ToInt64(sample)
		}
		return int64(456)
	case spec.KindU8:
		if sample != "" {
			return cast.ToUint8(sample)
		}
		return uint8(123)
	case spec.KindU16:
		if sample != "" {
			return cast.ToUint16(sample)
		}
		return uint16(234)
	case spec.KindU32:
		if sample != "" {
			return cast.ToUint32(sample)
		}
		return uint32(345)
	case spec.KindU64:
		if sample != "" {
			return cast.ToUint64(sample)
		}
		return uint64(456)
	case spec.KindList:
		return []interface{}{exampleValue(t.ItemType, sample, includeOptional)}
	case spec.KindMap:
		keyValue := fmt.Sprintf("%v", exampleValue(t.KeyType, sample, includeOptional))
		return map[string]interface{}{
			keyValue: exampleValue(t.ValueType, sample, includeOptional),
		}
	case spec.KindRaw:
	case spec.KindString:
		if sample != "" {
			return sample
		}
		return "string"
	case spec.KindType:
		return generatePayload(t.Type, includeOptional)
	case spec.KindUnion:
		ut := t.Union.Types[0]
		return exampleValue(ut, sample, includeOptional)
	}
	return nil
}

func getSample(a *spec.Annotated) string {
	sample := ""
	if annon, ok := a.Annotation("example"); ok {
		if arg, ok := annon.Argument("value"); ok {
			sample = arg.ValueString()
		}
	}
	return sample
}

func printPayload(payload interface{}) {
	jsonBytes, _ := json.MarshalIndent(payload, "", "  ")
	fmt.Println(string(jsonBytes))
}
