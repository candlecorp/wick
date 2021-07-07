.PHONY: all codegen build clean doc test

# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

SCHEMA_DIR := ./schemas
COMPONENTS_DIR := ./src/components
GENERATED_MODULE := ./src/generated.rs

# Get list of WIDL files in SCHEMA_DIR
SCHEMAS=$(wildcard ${SCHEMA_DIR}/*.widl)

# Translate a list of WIDL files to GENERATED_DIR/file.rs (transposing hyphens to underscores)
GENERATED_COMPONENTS := $(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${COMPONENTS_DIR}%,$(subst -,_,$(SCHEMAS))))
# Strip the directory and extension of the generated components to get their rust module names
GENERATED_FS_NAMES := $(patsubst %.rs,%,$(notdir $(GENERATED_COMPONENTS)))
# Generate WIDL import statements for each of the found WIDL files
GENERATED_WIDL_IMPORTS := $(patsubst $(SCHEMA_DIR)/%,import * from \"%\"\n,$(patsubst %.widl,%,$(SCHEMAS)))

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

RUSTFMT=rustfmt --edition=2018

# Generated files
GENERATED_FILES:=src/generated.rs
# The source files for the wapc module
SOURCE_FILES:=$(wildcard src/*.rs)

# The header to prepend to generated files
GENERATED_HEADER := /* This is generated, do not edit by hand */

# Files to clean up on make clean
CLEAN_FILES := $(GENERATED_FILES) $(BUILD_ARTIFACT) $(SIGNED_ARTIFACT)
CLEAN_DIRS :=

$(SIGNED_ARTIFACT): $(BUILD_ARTIFACT) ./interface.json
	vinoc sign $(BUILD_ARTIFACT) interface.json --ver=$(CRATE_VERSION) --rev=0
	vinoc inspect $(SIGNED_ARTIFACT)
#		 $(INPUTS) $(OUTPUTS)

# Defines rules like the following for each schema found :
# src/components/my_components.rs: schemas/my-component.widl /src/components
define WIDL_CODEGEN
$(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${COMPONENTS_DIR}%,$(subst -,_,$(1)))): $1 ${COMPONENTS_DIR}
	vino-codegen rust wapc-component $(notdir $(basename $(subst -,_,$(1)))) -o $$@
	rustfmt $$@
endef

# Call the above rule generator for each schema file
$(foreach schema,$(SCHEMAS),$(eval $(call WIDL_CODEGEN,$(schema))))

$(COMPONENTS_DIR):
	mkdir $@

$(ARTIFACT_DIR):
	mkdir -p $(ARTIFACT_DIR)

$(GENERATED_MODULE): $(SCHEMAS)
	vino-codegen rust wapc-integration $(SCHEMA_DIR) -f -o $@
	rustfmt $@

./interface.json: $(SCHEMAS)
	vino-codegen json interface $(SCHEMA_DIR) -o interface.json -f

./src/lib.rs: $(SCHEMAS)
	vino-codegen rust wapc-lib $(SCHEMA_DIR) -o $@
	rustfmt $@


$(BUILD_ARTIFACT): ./src/lib.rs $(ARTIFACT_DIR) $(SOURCE_FILES) $(GENERATED_MODULE) $($(GENERATED_COMPONENTS))
	cargo build --target wasm32-unknown-unknown --release
	cp $(WORKSPACE_ROOT)/target/wasm32-unknown-unknown/release/$(CRATE_FS_NAME).wasm build/

all: build

build: $(BUILD_ARTIFACT)

clean:
	rm -f $(CLEAN_FILES)
	# rmdir $(CLEAN_DIRS)

codegen: ./interface.json ./src/lib.rs $(GENERATED_MODULE)

doc:

test: build
	cargo test