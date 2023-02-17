package main

import (
	"context"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"go.uber.org/zap/zapcore"

	"github.com/nanobus/nanobus/pkg/engine"
	"github.com/nanobus/nanobus/pkg/handler"
)

type TestDefinition struct {
	file   string
	action string
	input  interface{}
	output interface{}
}

func TestInvoke(t *testing.T) {
	ctx := context.Background()
	tests := []TestDefinition{
		{
			file:   "std/log.yaml",
			action: "test::test",
			input:  nil,
			output: nil,
		},
		{
			file:   "std/decode.yaml",
			action: "test::test",
			input:  map[string]interface{}{"data": "{\"value\":\"test\"}"},
			output: map[string]interface{}{"value": "test"},
		},
		{
			file:   "std/expr.yaml",
			action: "test::value",
			input:  map[string]interface{}{"data": "some_val"},
			output: "some_val",
		},
		{
			file:   "std/expr.yaml",
			action: "test::data",
			input:  map[string]interface{}{"data": "some_val"},
			output: "some_val",
		},
		{
			file:   "std/filter.yaml",
			action: "test::test",
			input:  map[string]interface{}{"condition": true},
			output: "return value",
		},
		{
			file:   "std/filter.yaml",
			action: "test::test",
			input:  map[string]interface{}{"condition": false},
			output: nil,
		},
		{
			file:   "std/http.yaml",
			action: "test::test",
			input:  nil,
			output: nil,
		},
		{
			file:   "std/http.yaml",
			action: "test::decode",
			input:  nil,
			output: map[string]interface{}{"value": "test-value"},
		},
		{
			file:   "std/jq.yaml",
			action: "test::jqtest",
			input:  nil,
			output: "test-value",
		},
		// {
		// 	file:   "postgres/exec.yaml",
		// 	action: "test::testexec",
		// 	input:  nil,
		// 	output: "some_val",
		// },
		{
			file:   "blob/write.yaml",
			action: "test::testwrite",
			input:  nil,
			output: "test-value",
		},
	}
	cwd, _ := os.Getwd()
	for _, test := range tests {
		config := engine.Info{
			Mode:     engine.ModeInvoke,
			Target:   cwd + "/config/" + test.file,
			LogLevel: zapcore.DebugLevel,
		}
		e, err := engine.Start(ctx, &config)
		assert.NoError(t, err)
		action := handler.Handler{}
		action.FromString(test.action)
		fmt.Printf("testing action %s\n", action)
		result, err := e.InvokeUnsafe(action, test.input)
		assert.NoError(t, err)
		assert.Equal(t, test.output, result)
	}
}
