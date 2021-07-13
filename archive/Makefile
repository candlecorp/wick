.PHONY: all codegen install build clean test update-lint build-docker

# Enforce bash as the shell for consistency
SHELL := bash
# Use bash strict mode
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

CRATES_DIR := ./crates
ROOT := $(shell pwd)

# Get list of projects that have makefiles
MAKEFILE_PROJECTS=$(wildcard ${CRATES_DIR}/*/Makefile)

DOCKERFILES=$(wildcard docker/*/Dockerfile)

TEST_WASM=crates/test-wapc-component/build/test_component_s.wasm

BINS=vino vinoc vino-collection-inmemory vow

all: build

# Defines rules for each of the BINS to copy them into build/local
define COPY_BIN
$(1): build
	cp target/debug/$$@ build/local
endef

# Call the above rule generator for each BIN file
$(foreach bin,$(BINS),$(eval $(call COPY_BIN,$(bin))))

codegen:
	@for project in $(MAKEFILE_PROJECTS); do \
		cd `dirname $$project`; \
		echo "## Generating code for $$project"; \
		make codegen; \
		cd $(ROOT); \
	done

clean:
	@for project in $(MAKEFILE_PROJECTS); do \
		cd `dirname $$project`; \
		make clean; \
		cd $(ROOT); \
	done

install: $(BINS)
	cargo build --workspace
	cp build/local/* ~/.cargo/bin/

build: ./build/local codegen
	cargo build --workspace

$(TEST_WASM):
	cd crates/test-wapc-component && make && cd $(ROOT)

build-release: ./build/local
	cargo build --workspace --release

./build/local:
	mkdir -p ./build/local

test: $(TEST_WASM)
	cargo test --workspace

update-lint:
	@echo Checking git status...
	@[[ -z `git status -s` ]]
	@echo Good to go
	npm run update-lint

TAG:=registry.lan:5000/vino

build-docker: $(DOCKERFILES)
	for file in $(DOCKERFILES); do \
		docker build $$(dirname $$file) -t $(TAG)/$$(basename $$(dirname $$file)); \
		docker push $(TAG)/$$(basename $$(dirname $$file)); \
	done

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
