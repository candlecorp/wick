package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/alecthomas/kong"

	"github.com/nanobus/nanobus/engine"
)

var (
	Version = "edge"
	Commit  = "~~~~~"
	Date    = "~~~~~"
)

type Context struct{}

var commands struct {
	// Run
	Run    runCmd    `cmd:"" help:"Runs NanoBus"`
	Invoke invokeCmd `cmd:"" help:"Runs a single invocation using data from the command line"`
	// Version prints out the version of this program and runtime info.
	Version versionCmd `cmd:"Display version information"`
}

func main() {
	ctx := kong.Parse(&commands)
	// Call the `Run` method of the selected parsed command.
	err := ctx.Run(&Context{})
	ctx.FatalIfErrorf(err)
}

type runCmd struct {
	BusFile string   `arg:"" default:"bus.yaml"`
	Args    []string `arg:"" optional:""`
}

func (c *runCmd) Run() error {
	engine.Start(&engine.Info{
		Mode:    engine.ModeService,
		BusFile: c.BusFile,
		Process: c.Args,
	})
	return nil
}

type invokeCmd struct {
	BusFile   string `arg:"" required:""`
	Namespace string `required:""`
	Service   string `optional:""`
	Operation string `required:""`
	EntityID  string `name:"id" optional:""`
	Input     string `arg:"" optional:"" type:"existingfile"`
	Pretty    bool   `arg:"" default:"false"`
}

func (c *invokeCmd) Run() error {
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

	info := engine.Info{
		Mode:      engine.ModeInvoke,
		BusFile:   c.BusFile,
		Namespace: c.Namespace,
		Service:   c.Service,
		Operation: c.Operation,
		EntityID:  c.EntityID,
		Input:     input,
	}
	if err := engine.Start(&info); err != nil {
		os.Exit(1)
	}
	result := info.Output

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
