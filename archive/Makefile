.PHONY: all codegen build clean test update-lint

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