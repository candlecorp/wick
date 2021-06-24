.PHONY: all codegen build clean doc test

# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

SCHEMA_DIR := ./schemas
COMPONENTS_DIR := ./src/components
GENERATED_DIR := $(COMPONENTS_DIR)/generated
GENERATED_MODULE_RS:=$(GENERATED_DIR)/mod.rs
COMPONENTS_FACTORY_RS:=$(COMPONENTS_DIR)/mod.rs

# Get list of WIDL files in SCHEMA_DIR
SCHEMAS=$(wildcard ${SCHEMA_DIR}/*.widl)

# Translate a list of WIDL files to GENERATED_DIR/file.rs (transposing hyphens to underscores)
GENERATED_COMPONENTS := $(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${GENERATED_DIR}%,$(subst -,_,$(SCHEMAS))))
# Strip the directory and extension of the generated components to get their rust module names
GENERATED_FS_NAMES := $(patsubst %.rs,%,$(notdir $(GENERATED_COMPONENTS)))
# Generate module import statements for the generated component modules
GENERATED_MOD_IMPORTS := $(patsubst $(GENERATED_DIR)/%,pub (crate) mod %;,$(patsubst %.rs,%,$(GENERATED_COMPONENTS)))
# Generate WIDL import statements for each of the found WIDL files
GENERATED_WIDL_IMPORTS := $(patsubst $(SCHEMA_DIR)/%,import * from \"%\"\n,$(patsubst %.widl,%,$(SCHEMAS)))

# Temporary file to house the WIDL imports
TMP_SCHEMAS_FILE := ./.all_schemas.widl

# The header to prepend to generated files
GENERATED_HEADER := /* This is generated, do not edit by hand */

# Files to clean up on make clean
CLEAN_FILES := $(GENERATED_COMPONENTS) $(GENERATED_MODULE_RS) $(COMPONENTS_FACTORY_RS)
CLEAN_DIRS := $(GENERATED_DIR)

all: codegen build

# Defines rules like the following for each schema found :
# src/components/generated/my_components.rs: schemas/my-component.widl | /src/components/generated
define WIDL_CODEGEN
$(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${GENERATED_DIR}%,$(subst -,_,$(1)))): $1 | ${GENERATED_DIR}
endef

# Call the above rule generator for each schema file
$(foreach schema,$(SCHEMAS),$(eval $(call WIDL_CODEGEN,$(schema))))

$(GENERATED_MODULE_RS):
	printf "$(GENERATED_HEADER)\\n$(GENERATED_MOD_IMPORTS)" > $@

$(GENERATED_DIR):
	mkdir $@

$(GENERATED_COMPONENTS):
	vino-codegen rust "$<" -t provider-component >$(subst -,_,$@)

$(COMPONENTS_FACTORY_RS): $(SCHEMAS)
	touch $(TMP_SCHEMAS_FILE)
	printf "$(GENERATED_WIDL_IMPORTS)" > $(TMP_SCHEMAS_FILE)
	vino-codegen rust "$(TMP_SCHEMAS_FILE)" -t provider-integration -r ${SCHEMA_DIR} >$@
	rm $(TMP_SCHEMAS_FILE)


codegen:
	make $(GENERATED_COMPONENTS)
	make $(GENERATED_MODULE_RS)
	make $(COMPONENTS_FACTORY_RS)
	cargo fmt

build: codegen
	cargo build --release

clean:
	rm -f $(CLEAN_FILES)
	rmdir $(CLEAN_DIRS)

doc:

test: codegen
	cargo test