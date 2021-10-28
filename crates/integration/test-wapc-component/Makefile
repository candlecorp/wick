.DEFAULT_GOAL:=all

.PHONY: all codegen build clean doc test help list

ifdef OS
_OS := $(OS)
else
_OS := ""
endif

ifeq ($(_OS),Windows_NT)
# Use powershell on Windows
SHELL := powershell.exe
# mkdir -p on windows throws an error if the directory exists, -Force suppresses it.
MKDIR := mkdir -Force -p
.SHELLFLAGS := -NoProfile -Command
else
# Enforce bash as the shell for consistency
SHELL := bash
MKDIR := mkdir -p
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
endif

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
# Directory to store the build artifacts
ARTIFACT_DIR:=build
# Path to copy the build artifact to
BUILD_ARTIFACT:=$(ARTIFACT_DIR)/$(CRATE_FS_NAME).wasm
# Path for the signed artifact
SIGNED_ARTIFACT:=$(ARTIFACT_DIR)/$(CRATE_FS_NAME)_s.wasm
# Path to the interface JSON
INTERFACE:=./interface.json

VINOC=cargo run --manifest-path ../../../Cargo.toml -p vinoc

# Files to clean on make clean
CLEAN_FILES := $(GENERATED_MODULE) $(BUILD_ARTIFACT) $(SIGNED_ARTIFACT) ./src/components/mod.rs $(INTERFACE)

$(SIGNED_ARTIFACT): $(BUILD_ARTIFACT) $(INTERFACE)
	@echo Signing $(BUILD_ARTIFACT)
	@$(VINOC) -- sign $(BUILD_ARTIFACT) $(INTERFACE) --ver=$(CRATE_VERSION) --rev=0
	@echo Created $(SIGNED_ARTIFACT)

# Defines rules like the following for each schema found :
# src/components/my_components.rs: schemas/my-component.widl
define WIDL_CODEGEN
$(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${COMPONENTS_DIR}%,$(subst -,_,$(1)))): $1
	@$$(shell $$(MKDIR) $$(dir $$@))
	@echo Generating $$@
	@vino-codegen rust wapc-component $(notdir $(basename $(subst -,_,$(1)))) -o $$@
	@rustfmt $$@
endef

# Call the above rule generator for each schema file
$(foreach schema,$(SCHEMAS),$(eval $(call WIDL_CODEGEN,$(schema))))

$(GENERATED_MODULE): $(SCHEMAS)
	@echo Generating $@
	@vino-codegen rust wapc-integration $(SCHEMA_DIR) -f -o $@
	@rustfmt $@

$(COMPONENTS_DIR):
	@echo Making directory \"$@\"
	$(shell $(MKDIR) $@)

$(ARTIFACT_DIR):
	@echo Making directory \"$@\"
	$(shell $(MKDIR) $@)

$(COMPONENTS_MODULE): $(SCHEMAS) $(COMPONENTS_DIR)
	@echo Building $(COMPONENTS_MODULE)
	@vino-codegen rust wapc-component-module $(SCHEMA_DIR) -o $@ -f

$(BUILD_ARTIFACT): $(wildcard src/*.rs) ./src/lib.rs $(ARTIFACT_DIR) $(COMPONENTS_MODULE) $(GENERATED_MODULE) $(GENERATED_COMPONENTS)
	@echo Building artifact
	cargo build --target wasm32-unknown-unknown --release
	@echo Copying binary to $(BUILD_ARTIFACT)
	@cp $(WORKSPACE_ROOT)/target/wasm32-unknown-unknown/release/$(CRATE_FS_NAME).wasm build/

##@ Targets

$(INTERFACE): $(SCHEMAS) ## Create an interface.json from the project's schemas
	@echo Building $@ from schemas in $(SCHEMA_DIR)
	@vino-codegen json interface "$(CRATE_NAME)" $(SCHEMA_DIR) -o $@ -f

all: $(SIGNED_ARTIFACT) ## Make and sign the wasm binary

clean: ## Clean the generated files
	@rm -f $(CLEAN_FILES)

codegen: $(INTERFACE)  $(GENERATED_COMPONENTS) $(COMPONENTS_MODULE) $(GENERATED_MODULE) ./src/lib.rs ## Generate code from schemas

doc: ## Generate documentation
	@echo Unimplemented

test: build ## Run tests
	cargo test

##@ Helpers

list: ## Print schemas
	@echo $(SCHEMAS)

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z0-9_\-.*]+:.*?##/ { printf "  \033[36m%-32s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
