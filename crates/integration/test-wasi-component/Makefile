.DEFAULT_GOAL:=all

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
GENERATED_MODULE := ./src/components.rs

# Get list of WIDL files in SCHEMA_DIR
SCHEMAS=$(wildcard ${SCHEMA_DIR}/*.widl)

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
# All *.rs files in src/
RUST_SOURCES:=$(wildcard src/*.rs) $(wildcard src/*/*.rs)

TARGET=wasm32-wasi

VINOC=cargo run --manifest-path ../../../Cargo.toml -p vinoc --
CODEGEN:=vino-codegen

# Files to clean on make clean
CLEAN_FILES := $(GENERATED_MODULE) $(BUILD_ARTIFACT) $(SIGNED_ARTIFACT) ./src/components/mod.rs $(INTERFACE)

$(SIGNED_ARTIFACT): $(BUILD_ARTIFACT) $(INTERFACE)
	@echo Signing $(BUILD_ARTIFACT)
	@$(VINOC) sign $(BUILD_ARTIFACT) $(INTERFACE) --ver=$(CRATE_VERSION) --rev=0
	@echo Created $(SIGNED_ARTIFACT)

$(GENERATED_MODULE): $(INTERFACE)
	@echo Generating $@
	@$(CODEGEN) rust integration $(INTERFACE) --type wasm -f -o $@
	@rustfmt $@

$(COMPONENTS_DIR):
	@echo Making directory \"$@\"
	$(shell $(MKDIR) $@)

$(ARTIFACT_DIR):
	@echo Making directory \"$@\"
	$(shell $(MKDIR) $@)

components: $(INTERFACE) $(COMPONENTS_DIR)
	@echo Generating component stubs
	@$(CODEGEN) rust component $(INTERFACE) --type wasm -a -o $(COMPONENTS_DIR)

$(BUILD_ARTIFACT): $(ARTIFACT_DIR) components $(GENERATED_MODULE) $(RUST_SOURCES)
	@echo Building artifact
	cargo build --target $(TARGET) --release
	@echo Copying binary to $(BUILD_ARTIFACT)
	@cp $(WORKSPACE_ROOT)/target/$(TARGET)/release/$(CRATE_FS_NAME).wasm build/

##@ Targets

$(INTERFACE): $(SCHEMAS) ## Create an interface.json from the project's schemas
	@echo Building $@ from schemas in $(SCHEMA_DIR)
	@$(CODEGEN) json interface "$(CRATE_NAME)" $(SCHEMA_DIR) -o $@ -f

.PHONY: all
all: $(SIGNED_ARTIFACT) ## Make and sign the wasm binary

.PHONY: clean
clean: ## Clean the generated files
	@rm -f $(CLEAN_FILES)

.PHONY: codegen
codegen: $(INTERFACE) components $(GENERATED_MODULE) ./src/lib.rs ## Generate code from schemas

.PHONY: doc
doc: ## Generate documentation
	@echo Unimplemented

.PHONY: test
test: build ## Run tests
	cargo test

##@ Helpers

.PHONY: list
list: ## Print schemas
	@echo $(SCHEMAS)

.PHONY: help
help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z0-9_\-.*]+:.*?##/ { printf "  \033[36m%-32s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
