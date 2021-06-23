.PHONY: clean doc test

# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

# Name of the package from Cargo.toml
CRATE_NAME:=$(shell tomlq -f Cargo.toml package.name)
# Name with hyphens substituted with underscores
CRATE_FS_NAME:=$(subst -,_,$(CRATE_NAME))
# Version from Cargo.toml
CRATE_VERSION:=$(shell tomlq -f Cargo.toml package.version)
# Get the root director (respecting cargo workspaces)
WORKSPACE_ROOT:=$(shell cargo metadata --no-deps --format-version 1 -q | jq -r '.workspace_root')
# Directory to copy artifacts to
ARTIFACT_DIR:=build
# Path to copy artifact to
BUILD_ARTIFACT:=$(ARTIFACT_DIR)/$(CRATE_FS_NAME).wasm
# Path to copy artifact to
SIGNED_ARTIFACT:=$(ARTIFACT_DIR)/$(CRATE_FS_NAME)_s.wasm
# Path to the schema widl
SCHEMA_FILE:=schema.widl
# Generated files
GENERATED_FILES:=src/generated.rs
# The input ports from schema
INPUTS:=$(shell widl2json $(SCHEMA_FILE) --condensed | jq -r '.types | map(select(.name == "Inputs").fields[]) | map("--input-port=\(.name)")| join(",")')
# The output ports from schema
OUTPUTS:=$(shell widl2json $(SCHEMA_FILE) --condensed | jq -r '.types | map(select(.name == "Outputs").fields[]) | map("--output-port=\(.name)")| join(",")')
# All source rust files
SOURCE_FILES:=$(wildcard src/*.rs)

CLEANUP_FILES=$(GENERATED_FILES) $(BUILD_ARTIFACT) $(SIGNED_ARTIFACT)

$(SIGNED_ARTIFACT): $(BUILD_ARTIFACT)
	wash claims sign $(BUILD_ARTIFACT) --name=$(CRATE_NAME) --ver=$(CRATE_VERSION) \
		--rev=0 $(INPUTS) $(OUTPUTS)

$(BUILD_ARTIFACT): $(ARTIFACT_DIR) $(SOURCE_FILES) $(GENERATED_FILES)
	cargo build --target wasm32-unknown-unknown --release
	cp $(WORKSPACE_ROOT)/target/wasm32-unknown-unknown/release/$(CRATE_FS_NAME).wasm build/

$(GENERATED_FILES): codegen.yaml $(SCHEMA_FILE)
	wapc generate $<

$(ARTIFACT_DIR):
	mkdir -p $(ARTIFACT_DIR)

clean:
	rm -f $(CLEANUP_FILES)

doc:

test:
	cargo test
