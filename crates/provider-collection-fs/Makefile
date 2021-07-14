.PHONY: all codegen build clean doc test

# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

SCHEMA_DIR := ./schemas
COMPONENTS_DIR := ./src/components
GENERATED_RS:=./src/generated.rs
COMPONENTS_RS:=$(COMPONENTS_DIR)/mod.rs

# Name of the package from Cargo.toml
CRATE_NAME:=$(shell tomlq -f Cargo.toml package.name)

# Files to clean up on make clean
CLEAN_FILES := $(GENERATED_RS) $(COMPONENTS_RS)

all: codegen build

$(GENERATED_RS): ./interface.json
	vino-codegen rust wellknown-integration $< -f -o $@

$(COMPONENTS_RS): ./interface.json
	vino-codegen rust wellknown-component-module $< -f -o $@

codegen: $(GENERATED_RS) $(COMPONENTS_RS)
	cargo fmt

build: $(wildcard ./src/*.rs) $(GENERATED_RS) ./interface.json
	cargo build --release

clean:
	rm -f $(CLEAN_FILES)

doc:

test: codegen
	cargo test