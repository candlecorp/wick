
# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

CRATES_DIR := ./crates
ROOT := $(shell pwd)

# Get list of projects that have makefiles
MAKEFILE_PROJECTS=$(wildcard ${CRATES_DIR}/*/Makefile) $(wildcard ${CRATES_DIR}/integration/*/Makefile) $(wildcard ${CRATES_DIR}/interfaces/*/Makefile)

# Get list of root crates in $CRATES_DIR
ROOT_RUST_CRATES=$(wildcard ${CRATES_DIR}/*/Cargo.toml)

TEST_WASM_DIR=$(CRATES_DIR)/integration/test-wapc-component
TEST_WASM=$(TEST_WASM_DIR)/build/test_component_s.wasm

BINS=vino vinoc vino-collection-inmemory vow vino-authentication-inmemory vino-collection-fs vino-keyvalue-redis

.PHONY: all codegen install install-release  clean test update-lint build build-release wasm

all: build

# Defines rules for each of the BINS to copy them into build/local
define BUILD_BIN
$(1): build
	cp target/debug/$$@ build/local
endef

# Call the above rule generator for each BIN file
$(foreach bin,$(BINS),$(eval $(call BUILD_BIN,$(bin))))

codegen:
	@for project in $(MAKEFILE_PROJECTS); do \
		cd `dirname $$project`; \
		echo "## Generating code for $$project"; \
		make codegen; \
		cd $(ROOT); \
	done

readme:
	@for project in $(ROOT_RUST_CRATES); do \
		cd `dirname $$project`; \
		echo "## Generating README for $$project"; \
		cargo readme > README.md; \
		cd $(ROOT); \
	done

clean:
	@for project in $(MAKEFILE_PROJECTS); do \
		cd `dirname $$project`; \
		make clean; \
		cd $(ROOT); \
	done

install-release: $(BINS)
	cargo build --workspace --release
	cp build/local/* ~/.cargo/bin/

install: $(BINS)
	cargo build --workspace
	cp build/local/* ~/.cargo/bin/

build: ./build/local codegen
	cargo build --workspace --all

$(TEST_WASM):
	cd $(TEST_WASM_DIR) && make && cd $(ROOT)

build-release: ./build/local
	cargo build --workspace --release

./build/local:
	mkdir -p ./build/local

wasm: $(TEST_WASM)

test: $(TEST_WASM)
	cargo deny check licenses --hide-inclusion-graph
	cargo build --workspace # necessary to ensure binaries are built
	cargo test --workspace

update-lint:
	@echo Checking git status...
	@[[ -z `git status -s` ]]
	@echo Good to go
	npm run update-lint

build-cross-debug:
	rm -rf ./build; \
	mkdir ./build; \
	for file in $(DOCKERFILES); do \
		TARGET="$$(basename $$(dirname $$file))"; \
		mkdir ./build/$$TARGET; \
		cross build -p vino-cli --target $$TARGET; \
		cp target/$$TARGET/debug/vino build/$$TARGET/; \
		cross build -p vinoc --target $$TARGET; \
		cp target/$$TARGET/debug/vinoc build/$$TARGET/; \
		cross build -p vino-collection-inmemory --target $$TARGET; \
		cp target/$$TARGET/debug/vino-collection-inmemory build/$$TARGET/provider-collection-inmemory; \
	done
