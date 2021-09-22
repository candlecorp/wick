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

# Get component names from the interface definition JSON
COMPONENT_NAMES := $(shell cat interface.json | jq -r '.components[].name')

# Translate a list of component names to $COMPONENTS_DIR/file.rs (transposing hyphens to underscores)
GENERATED_COMPONENTS := $(patsubst %,$(COMPONENTS_DIR)/%.rs,$(subst -,_,$(COMPONENT_NAMES)))

all: build

# Defines rules like the following for each schema found :
# src/components/my_components.rs: interface.json /src/components
define COMPONENT_GEN
$(COMPONENTS_DIR)/$(subst -,_,$(1)).rs: interface.json ${COMPONENTS_DIR}
	vino-codegen rust provider-component $(1) -o $$@
	rustfmt $$@
endef

# Call the above rule generator for each schema file
$(foreach name,$(COMPONENT_NAMES),$(eval $(call COMPONENT_GEN,$(name))))

$(GENERATED_RS): ./interface.json
	vino-codegen rust wellknown-integration $< -f -o $@

$(COMPONENTS_DIR):
	mkdir $@

$(COMPONENTS_RS): ./interface.json $(COMPONENTS_DIR)
	vino-codegen rust wellknown-component-module $< -f -o $@

codegen: ${GENERATED_COMPONENTS} $(GENERATED_RS) $(COMPONENTS_RS)
	cargo fmt

build: $(wildcard ./src/*.rs) ${GENERATED_COMPONENTS} $(GENERATED_RS) ./interface.json
	cargo build --release

clean:
	rm -f $(CLEAN_FILES)

doc:

test: codegen
	cargo test