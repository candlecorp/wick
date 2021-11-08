.DEFAULT_GOAL:=help

# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

CRATES_DIR := ./crates
ROOT := $(shell pwd)

# Get list of root crates in $CRATES_DIR
ROOT_RUST_CRATES := $(dir $(wildcard ${CRATES_DIR}/*/Cargo.toml))

READMES := $(patsubst %,%README.md,$(ROOT_RUST_CRATES))

CLEAN_FILES := $(READMES)

##@ Main targets

.PHONY: all clean test update-lint build readmes

all: build  ## Alias for make build

${CRATES_DIR}/%/README.md:
	@echo "## Generating README for $(dir $@)"; \
	cd $(dir $@) && cargo readme > README.md

readmes: $(READMES) ## Generate the README.md files from rustdoc

clean: ## Remove generated files (run cargo clean to clean rust's cache)
	@rm -f $(CLEAN_FILES)

build: ## Build everything in the workspace
	cargo build --workspace

test: ## Test all crates in the workspace
	cargo deny check licenses --hide-inclusion-graph
	cargo doc --no-deps --workspace --all-features
	cargo test --workspace --all-features

update-lint: ## Update clippy lint definitions in sub crates
	npm run update-lint

##@ Helpers

.PHONY: help

list: ## Display supported images
	@ls Dockerfile.* | sed -nE 's/Dockerfile\.(.*)/\1/p' | sort

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z0-9_\-.*]+:.*?##/ { printf "  \033[36m%-32s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
