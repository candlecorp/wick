/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"reflect"

	"github.com/alecthomas/kong"
	"go.uber.org/zap/zapcore"

	"github.com/nanobus/nanobus/pkg/channel/metadata"
	"github.com/nanobus/nanobus/pkg/engine"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/logger"
	"github.com/nanobus/nanobus/pkg/oci"
	"github.com/nanobus/nanobus/pkg/runtime"
	"github.com/nanobus/nanobus/pkg/stream"
)

var (
	Version = "edge"
	Commit  = "~~~~~"
	Date    = "~~~~~"
)

type Context struct{}

var commands struct {
	DefaultRun defaultRunCmd `cmd:"" hidden:"" default:""`
	// Run starts an application from a local configuration or OCI image reference.
	Run runCmd `cmd:"run" help:"Runs a NanoBus application from a local configuration or OCI image reference"`
	// Invoke runs a single invocation using input from STDIN or a file.
	Invoke invokeCmd `cmd:"" help:"Runs a single invocation using input from STDIN or a file"`
	// Push packages and uploads the an application to an OCI registry.
	Push pushCmd `cmd:"" help:"Packages and pushes a NanoBus application to an OCI registry"`
	// Pull retrieves an application from an OCI registry without running it.
	Pull pullCmd `cmd:"" help:"Pulls a NanoBus application from an OCI registry without running it"`
	// Version prints out the version of this program and runtime info.
	Version versionCmd `cmd:"Display version information"`
}

func main() {
	ctx := kong.Parse(&commands)
	// Call the `Run` method of the selected parsed command.
	err := ctx.Run(&Context{})
	ctx.FatalIfErrorf(err)
}

type defaultRunCmd struct {
	DeveloperMode bool `name:"developer-mode" help:"Enables developer mode."`
	// Turns on debug logging.
	Debug bool `name:"debug" help:"Turns on debug logging"`
}

func (c *defaultRunCmd) Run() error {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// The default command currently does not accept flags.
	// This flag code still is here in case it does in the future.
	level := zapcore.InfoLevel
	if c.Debug {
		level = zapcore.DebugLevel
	}
	if _, err := engine.Start(ctx, &engine.Info{
		Mode:          engine.ModeService,
		BusFile:       "bus.yaml",
		ResourcesFile: "resources.yaml",
		PackageFile:   "",
		LogLevel:      level,
		DeveloperMode: c.DeveloperMode,
	}); err != nil {
		// Error is logged in `Start`.
		cancel()
		os.Exit(1)
	}

	return nil
}

type runCmd struct {
	Target        string `arg:"positional" optional:"" help:"The application configuration yaml file or OCI image reference."`
	DeveloperMode bool   `name:"developer-mode" help:"Enables developer mode."`
	// BusFile of the application as a configuration yaml file.
	BusFile string `name:"bus" default:"bus.yaml" help:"The application configuration yaml file."`
	// ResourcesFile is the resources configuration (e.g. databases, message brokers).
	ResourcesFile string `name:"resources" default:"resources.yaml" help:"The resources configuration"`
	// PackageFile is the resources configuration (e.g. databases, message brokers).
	PackageFile string `name:"package" optional:"" help:"The registry hosted pacakage containing a packaged NanoBus application"`
	// Turns on debug logging.
	Debug bool `name:"debug" help:"Turns on debug logging"`
	// Args are arguments passed to the application.
	Args []string `arg:"" optional:"" help:"Arguments to pass to the application"`
}

func (c *runCmd) Run() error {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var reference string
	if oci.IsImageReference(c.Target) {
		reference = c.Target
	}

	level := zapcore.InfoLevel
	if c.Debug {
		level = zapcore.DebugLevel
	}

	if _, err := engine.Start(ctx, &engine.Info{
		Mode:          engine.ModeService,
		BusFile:       c.BusFile,
		LogLevel:      level,
		ResourcesFile: c.ResourcesFile,
		PackageFile:   reference,
		DeveloperMode: c.DeveloperMode,
	}); err != nil {
		// Error is logged in `Start`.
		cancel()
		os.Exit(1)
	}

	return nil
}

type invokeCmd struct {
	DeveloperMode bool `name:"developer-mode" help:"Enables developer mode."`
	// Operation is the operation name.
	Operation string `arg:"" required:"" help:"The operation or function invoke"`
	// BusFile is the application configuration (not an OCI image reference).
	BusFile string `name:"bus" default:"bus.yaml" help:"The NanoBus application configuration"`
	// ResourcesFile is the resources configuration (e.g. databases, message brokers).
	ResourcesFile string `name:"resources" default:"resources.yaml" help:"The resources configuration"`
	// PackageFile is the resources configuration (e.g. databases, message brokers).
	PackageFile string `name:"package" optional:"" help:"The registry hosted pacakage containing a packaged NanoBus application"`
	// EntityID is the entity identifier to invoke.
	EntityID string `name:"id" optional:"" help:"The entity ID to invoke (e.g. actor ID)"`
	// Input is the file to use as JSON input.
	Input string `arg:"" optional:"" type:"existingfile" help:"File to use as input JSON data"`
	// Pretty is a flag to pretty print the JSON output.
	Pretty bool `name:"pretty" default:"false" help:"Pretty print the JSON output"`
	// Turns on debug logging.
	Debug bool `name:"debug" help:"Turns on debug logging"`
}

func (c *invokeCmd) Run() error {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	inputFile := os.Stdin
	if c.Input != "" {
		f, err := os.Open(c.Input)
		if err != nil {
			return err
		}
		defer f.Close()
	}
	inputBytes, err := io.ReadAll(inputFile)
	if err != nil {
		return fmt.Errorf("could not read stdin: %w", err)
	}

	var input map[string]interface{}
	if err := json.Unmarshal(inputBytes, &input); err != nil {
		return fmt.Errorf("could not parse stdin: %w", err)
	}

	var h handler.Handler
	if err := h.FromString(c.Operation); err != nil {
		return fmt.Errorf("invalid operation %q: %w", c.Operation, err)
	}
	level := zapcore.ErrorLevel
	if c.Debug {
		level = zapcore.DebugLevel
	}

	sink := &invokeSink{}
	ctx = stream.SinkNewContext(ctx, sink)

	info := engine.Info{
		Mode:          engine.ModeInvoke,
		BusFile:       c.BusFile,
		LogLevel:      level,
		ResourcesFile: c.ResourcesFile,
		PackageFile:   c.PackageFile,
		EntityID:      c.EntityID,
		DeveloperMode: c.DeveloperMode,
	}
	e, err := engine.Start(ctx, &info)
	if err != nil {
		// Error is logged in `Start`.
		cancel()
		os.Exit(1)
	}

	var result any
	result, err = e.InvokeUnsafe(h, input)
	if err != nil {
		logger.Error("error invoking operation", "error", err)
		return nil
	}

	if !isNil(result) {
		var jsonBytes []byte
		if c.Pretty {
			jsonBytes, err = json.MarshalIndent(result, "", "  ")
		} else {
			jsonBytes, err = json.Marshal(result)
		}
		if err != nil {
			return fmt.Errorf("error converting output to JSON: %w", err)
		}

		fmt.Println(string(jsonBytes))
	}

	return nil
}

type pushCmd struct {
	// BusFile is the application configuration (not an OCI image reference).
	BusFile string `type:"existingFile" arg:"" default:"bus.yaml" help:"The NanoBus application configuration"`
	// Registry is the OCI registry hostname:port.
	Registry string `optional:"" help:"The OCI registry hostname:port"`
	// Org is the OCI registry organization/project.
	Org string `optional:"" help:"The OCI registry organization/project"`
	// ApplicationID is the OCI application/repository.
	ApplicationID string `name:"application-id" optional:"" help:"The OCI application/repository"`
	// DryRun is a flag denoting to run only the package phase without uploading the OCI manifest.
	DryRun bool `name:"dry-run" default:"false" help:"Run only the package phase without uploading the OCI manifest"`
}

func (c *pushCmd) Run() error {
	busFile, err := os.Open(c.BusFile)
	if err != nil {
		return err
	}
	defer busFile.Close()

	absPath, err := filepath.Abs(c.BusFile)
	if err != nil {
		return err
	}
	baseDir := filepath.Dir(absPath)

	conf, err := runtime.LoadBusYAML(baseDir, busFile)
	if err != nil {
		return err
	}

	if conf.Package == nil {
		return errors.New("package is not defined in configuration")
	}

	registry := c.Registry
	if conf.Package != nil && conf.Package.Registry != nil && registry == "" {
		registry = *conf.Package.Registry
	}
	if registry == "" {
		registry = "reg.candle.run"
	}

	org := c.Org
	if conf.Package != nil && conf.Package.Org != nil && org == "" {
		org = *conf.Package.Org
	}
	if org == "" {
		return errors.New("organization/project is not defined")
	}

	applicationID := c.ApplicationID
	if conf.ID != "" && applicationID == "" {
		applicationID = conf.ID
	}
	if applicationID == "" {
		return errors.New("application id is not defined")
	}

	reference := fmt.Sprintf("%s/%s/%s:%s", registry, org, applicationID, conf.Version)
	if c.DryRun {
		fmt.Printf("Pushing %s (dry run)\n", reference)
	} else {
		fmt.Printf("Pushing %s\n", reference)
	}

	add := conf.Package.Add
	add = append(add, filepath.Clean(c.BusFile)+":"+oci.AppMediaType)

	return oci.Push(reference, ".", add, c.DryRun)
}

type pullCmd struct {
	// Reference is the full OCI image reference to pull.
	Reference string `arg:"" help:"The OCI image reference to pull"`
}

func (c *pullCmd) Run() error {
	if _, err := oci.Pull(c.Reference, "."); err != nil {
		fmt.Printf("Error pulling image: %s\n", err)
		return err
	}

	return nil
}

type versionCmd struct{}

func (c *versionCmd) Run() error {
	println("version = " + Version)
	if Commit != "" {
		println("commit  = " + Commit)
	}
	if Date != "" {
		println("date    = " + Date)
	}

	return nil
}

type invokeSink struct {
	pretty bool
}

func (i *invokeSink) Next(data any, md metadata.MD) (err error) {
	var jsonBytes []byte
	if i.pretty {
		jsonBytes, err = json.MarshalIndent(data, "", "  ")
	} else {
		jsonBytes, err = json.Marshal(data)
	}
	if err != nil {
		return fmt.Errorf("error converting output to JSON: %w", err)
	}

	fmt.Println(string(jsonBytes))
	return nil
}

func (i *invokeSink) Complete() {}

func (i *invokeSink) Error(err error) {
	logger.Error(err.Error())
}

func isNil(val interface{}) bool {
	return val == nil ||
		(reflect.ValueOf(val).Kind() == reflect.Ptr &&
			reflect.ValueOf(val).IsNil())
}
