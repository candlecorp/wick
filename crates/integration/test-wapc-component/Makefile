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
COMPONENTS_MODULE := $(COMPONENTS_DIR)/mod.rs

# Get list of WIDL files in SCHEMA_DIR
SCHEMAS=$(wildcard ${SCHEMA_DIR}/*.widl)

# Translate a list of WIDL files to COMPONENTS_DIR/file.rs (transposing hyphens to underscores)
GENERATED_COMPONENTS := $(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${COMPONENTS_DIR}%,$(subst -,_,$(SCHEMAS))))

# Name of the package from Cargo.toml
CRATE_NAME:=$(shell tomlq -f Cargo.toml package.name)
# Name with hyphens substituted with underscores
CRATE_FS_NAME:=$(subst -,_,$(CRATE_NAME))
# Version from Cargo.toml
CRATE_VERSION:=$(shell tomlq -f Cargo.toml package.version)
# Get the root directory
WORKSPACE_ROOT:=.
# Directory to copy artifacts to
ARTIFACT_DIR:=build
# Path to copy artifact to
BUILD_ARTIFACT:=$(ARTIFACT_DIR)/$(CRATE_FS_NAME).wasm
# Path to copy artifact to
SIGNED_ARTIFACT:=$(ARTIFACT_DIR)/$(CRATE_FS_NAME)_s.wasm


# Files to clean up on make clean
CLEAN_FILES := $(GENERATED_MODULE) $(BUILD_ARTIFACT) $(SIGNED_ARTIFACT) ./src/components/mod.rs

$(SIGNED_ARTIFACT): $(BUILD_ARTIFACT) ./interface.json
	vinoc sign $(BUILD_ARTIFACT) interface.json --ver=$(CRATE_VERSION) --rev=0
	vinoc inspect $(SIGNED_ARTIFACT)

# Defines rules like the following for each schema found :
# src/components/my_components.rs: schemas/my-component.widl
define WIDL_CODEGEN
$(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${COMPONENTS_DIR}%,$(subst -,_,$(1)))): $1
	@mkdir -p $$(dir $$@)
	vino-codegen rust wapc-component $(notdir $(basename $(subst -,_,$(1)))) -o $$@
	rustfmt $$@
endef

# Call the above rule generator for each schema file
$(foreach schema,$(SCHEMAS),$(eval $(call WIDL_CODEGEN,$(schema))))

$(GENERATED_MODULE): $(SCHEMAS)
	vino-codegen rust wapc-integration $(SCHEMA_DIR) -f -o $@
	rustfmt $@

./interface.json: $(SCHEMAS)
	vino-codegen json interface "$(CRATE_NAME)" $(SCHEMA_DIR) -o interface.json -f

$(COMPONENTS_MODULE): $(SCHEMAS)
	@mkdir -p $(dir $@)
	vino-codegen rust wapc-component-module $(SCHEMA_DIR) -o $@ -f
	rustfmt $@

$(BUILD_ARTIFACT): $(wildcard src/*.rs) ./src/lib.rs $(COMPONENTS_MODULE) $(GENERATED_MODULE) $(GENERATED_COMPONENTS)
	@mkdir -p $(dir $@)
	cargo build --target wasm32-unknown-unknown --release
	cp $(WORKSPACE_ROOT)/target/wasm32-unknown-unknown/release/$(CRATE_FS_NAME).wasm build/

all: $(SIGNED_ARTIFACT)

clean:
	rm -f $(CLEAN_FILES)

codegen: ./interface.json  $(GENERATED_COMPONENTS) $(COMPONENTS_MODULE) $(GENERATED_MODULE) ./src/lib.rs

doc:

test: build
	cargo test