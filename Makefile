
# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --no-print-directory

CRATES_DIR := ./crates
ROOT := $(shell pwd)

# Get list of projects that have makefiles
MAKEFILES=$(wildcard ${CRATES_DIR}/*/*/Makefile)
MAKEFILE_PROJECTS=$(foreach makefile,$(MAKEFILES),$(dir $(makefile)))

# Get list of root crates in $CRATES_DIR
ROOT_RUST_CRATES=$(foreach crate,$(wildcard ${CRATES_DIR}/*/Cargo.toml),$(dir $(crate)))

TEST_WASM_DIR=$(CRATES_DIR)/integration/test-wapc-component
TEST_WASM=$(TEST_WASM_DIR)/build/test_component_s.wasm

TEST_WASI_DIR=$(CRATES_DIR)/integration/test-wasi-component
TEST_WASI=$(TEST_WASI_DIR)/build/test_wasi_component_s.wasm

BINS=vinoc vino vow vino-keyvalue-redis

RELEASE?=false
ARCH?=local

ifneq (,$(findstring pc-windows,$(ARCH))) # If arch is *pc-windows*
BIN_SUFFIX:=.exe
else
BIN_SUFFIX:=
endif

.PHONY: all
all: build

# Defines rules for each of the BINS to copy them into build/local
define BUILD_BIN
$(1): build
	cp target/debug/$$@ build/local
endef

# Call the above rule generator for each BIN file
$(foreach bin,$(BINS),$(eval $(call BUILD_BIN,$(bin))))

.PHONY: cleangen
cleangen:
	@for project in $(MAKEFILE_PROJECTS); do \
		echo "# Cleaning $$project"; \
		$(MAKE) -C $$project clean; \
		echo "# Generating code for $$project"; \
		$(MAKE) -C $$project codegen; \
	done

.PHONY: codegen
codegen:
	@for project in $(MAKEFILE_PROJECTS); do \
		echo "# Generating code for $$project"; \
		$(MAKE) -C $$project codegen; \
	done

.PHONY: readme
readme:
	@for project in $(ROOT_RUST_CRATES); do \
		cd $$project; \
		echo "# Generating README for $$project"; \
		cargo readme > README.md; \
		cd $(ROOT); \
	done

.PHONY: clean
clean:
	@rm -rf $(TEST_WASI) $(TEST_WASM)
	@rm -rf ./build/*
	@for project in $(MAKEFILE_PROJECTS); do \
		echo "# Cleaning $$project"; \
		$(MAKE) -C $$project clean; \
	done
	cargo clean

.PHONY: install-release
install-release: $(BINS)
	cargo build --workspace --release
	cp build/local/* ~/.cargo/bin/

.PHONY: install
install: $(BINS)
	cargo build --workspace
	cp build/local/* ~/.cargo/bin/

.PHONY: build
build: ./build/local codegen
	cargo build --workspace --all

$(TEST_WASM):
	$(MAKE) -C $(TEST_WASM_DIR)

$(TEST_WASI):
	$(MAKE) -C $(TEST_WASI_DIR)

./build/$(ARCH):
	mkdir -p $@

.PHONY: wasm
wasm: $(TEST_WASM) $(TEST_WASI)

.PHONY: test
test: codegen wasm
	cargo deny check licenses --hide-inclusion-graph
	cargo build --workspace # necessary to ensure binaries are built
	cargo test --workspace

.PHONY: integration
test-integration: $(TEST_WASM)
	cargo deny check licenses --hide-inclusion-graph
	cargo build --workspace # necessary to ensure binaries are built
	cargo test --workspace --features test-integration --features vino-lattice/test-integration --features vino-provider-lattice/test-integration --features vino-runtime/test-integration --features vino-keyvalue-redis/test-integration

.PHONY: update-lint
update-lint:
	npm run update-lint

.PHONY: build-tag
build-tag:
ifeq ($(shell git status -s),)
	@echo Tagging build-$$(date "+%Y-%m-%d")
	@git tag build-$$(date "+%Y-%m-%d") -f
else
	@echo "Check in changes before making a build tag."
endif

.PHONY: bins
bins: ./build/$(ARCH)
	@echo "Building ARCH=$(ARCH) RELEASE=$(RELEASE)"
	@rm -rf ./build/$(ARCH)/*
ifeq ($(ARCH),local)
ifeq ($(RELEASE),true)
	cargo build --release $(foreach bin,$(BINS),-p $(bin))
	cp $(foreach bin,$(BINS),./target/release/$(bin)$(BIN_SUFFIX)) ./build/$(ARCH)
else
	cargo build $(foreach bin,$(BINS),-p $(bin))
	cp $(foreach bin,$(BINS),./target/debug/$(bin)$(BIN_SUFFIX)) ./build/$(ARCH)
endif
else
ifeq ($(RELEASE),true)
ifeq ($(ARCH),x86_64-pc-windows-gnu)
	CARGO_PROFILE_RELEASE_LTO=false cross build --target $(ARCH) --release $(foreach bin,$(BINS),-p $(bin))
else
	cross build --target $(ARCH) --release $(foreach bin,$(BINS),-p $(bin))
endif
	cp $(foreach bin,$(BINS),./target/$(ARCH)/release/$(bin)$(BIN_SUFFIX)) ./build/$(ARCH)
else
	cross build --target $(ARCH) $(foreach bin,$(BINS),-p $(bin))
	cp $(foreach bin,$(BINS),./target/$(ARCH)/debug/$(bin)$(BIN_SUFFIX)) ./build/$(ARCH)
endif
endif

.PHONY: build-cross-debug
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
	done
