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

# Get list of WIDL files in SCHEMA_DIR
SCHEMAS=$(wildcard ${SCHEMA_DIR}/*.widl)
# Name of the package from Cargo.toml
CRATE_NAME:=$(shell tomlq -f Cargo.toml package.name)
# Translate a list of WIDL files to COMPONENTS_DIR/file.rs (transposing hyphens to underscores)
USER_COMPONENTS := $(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${COMPONENTS_DIR}%,$(subst -,_,$(SCHEMAS))))

# The header to prepend to generated files
GENERATED_HEADER := /* This is generated, do not edit by hand */

# Files to clean up on make clean
CLEAN_FILES :=  $(COMPONENTS_RS) $(GENERATED_RS)

RUSTFMT=rustfmt --edition=2018

all: codegen build

# Defines rules like the following for each schema found :
# src/components/my_components.rs: schemas/my-component.widl | /src/components
define WIDL_CODEGEN
# Generates component boilerplate for schemas that don't have a component module yet.
$(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${COMPONENTS_DIR}%,$(subst -,_,$(1)))): $1 | ${COMPONENTS_DIR}
	vino-codegen rust provider-component $(notdir $(basename $(subst -,_,$(1)))) -o $$@
	rustfmt $$@
endef

# Call the above rule generator for each schema file
$(foreach schema,$(SCHEMAS),$(eval $(call WIDL_CODEGEN,$(schema))))

$(COMPONENTS_RS): $(SCHEMAS)
	@vino-codegen rust provider-component-module $(SCHEMA_DIR) -f -o $@

$(COMPONENTS_DIR):
	@mkdir $@

./interface.json: $(SCHEMAS)
	@vino-codegen json interface "$(CRATE_NAME)" $(SCHEMA_DIR) -f -o $@

$(GENERATED_RS): $(SCHEMAS)
	@vino-codegen rust provider-integration $(SCHEMA_DIR) -f -o $@

codegen: $(USER_COMPONENTS)  $(GENERATED_RS) $(COMPONENTS_RS)
	@cargo fmt

build: $(wildcard ./src/*.rs) $(GENERATED_RS) $(USER_COMPONENTS) $(COMPONENTS_RS) ./interface.json
	@cargo build --release

clean:
	@rm -f $(CLEAN_FILES)

doc:

test: codegen
	@cargo test