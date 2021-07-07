.PHONY: all codegen build clean test update-lint build-docker

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

all: codegen build

codegen:
	for project in $(MAKEFILE_PROJECTS); do \
		cd `dirname $$project`; \
		make codegen; \
		cd $(ROOT); \
	done


clean:
	for project in $(MAKEFILE_PROJECTS); do \
		cd `dirname $$project`; \
		make clean; \
		cd $(ROOT); \
	done

build:
	cargo build --workspace
	cd crates/test-wapc-component && make && cd $(ROOT)

test: build
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

build-local:
	rm -rf ./build/local; \
	mkdir ./build/local; \
	cargo build -p vino-cli ; \
	cp target/debug/vino build/local/; \
	cargo build -p vinoc; \
	cp target/debug/vinoc build/local/; \
	cargo build -p vino-collection-inmemory; \
	cp target/debug/vino-collection-inmemory build/local/; \

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
