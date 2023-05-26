
nextest := `command -v cargo-nextest >/dev/null && echo true || echo false`
rustup := `command -v rustup >/dev/null && echo true || echo false`

set dotenv-load
set export

wasm:
	just crates/integration/test-baseline-component/build
	just crates/integration/test-http-trigger-component/build
	just crates/integration/test-cli-trigger-component/build
	just crates/integration/test-cli-with-db/build

debug-wasm:
	just crates/integration/test-baseline-component/debug
	just crates/integration/test-http-trigger-component/debug
	just crates/integration/test-cli-trigger-component/debug
	just crates/integration/test-cli-with-db/debug

clean:
	cargo clean
	rm -rf node_modules
	just crates/wick/wick-config/clean
	just crates/integration/test-baseline-component/clean
	just crates/integration/test-http-trigger-component/clean
	just crates/integration/test-cli-trigger-component/clean
	just crates/integration/test-cli-with-db/clean


codegen:
	just crates/wick/wick-config/codegen

test: codegen early-errors wasm unit-tests

install-debug:
	cargo install --path crates/bins/wick --debug

install:
	cargo install --path crates/bins/wick

early-errors: licenses
	cargo +nightly fmt --check
	cargo clippy --workspace --bins

licenses:
	cargo deny --workspace check licenses  --config etc/deny.toml --hide-inclusion-graph

unit-tests: codegen-tests
	if {{nextest}} = "true"; then \
	  cargo nextest run -E 'not (test(slow_test) | test(integration_test))'; \
	else \
	  cargo test --workspace -- --skip integration_test --skip slow_test --skip codegen-test --test-threads=6; \
	fi

codegen-tests:
	just tests/codegen-tests/codegen
	just tests/codegen-tests/test

ci-tests: wasm
	just unit-tests

integration: integration-setup && integration-teardown
	just integration-tests

integration-tests: codegen-tests
	if {{nextest}} = "true"; then \
	  cargo nextest run -E 'not (test(slow_test))'; \
	else \
	  cargo test --workspace -- --skip slow_test --test-threads=6; \
	fi
	cargo test --manifest-path tests/template/Cargo.toml
	just wick-db-tests

wick-db-tests:
  cargo run -p wick-cli -- test ./examples/db/flow-with-postgres.wick
  cargo run -p wick-cli -- test ./examples/db/postgres-numeric.wick

all-tests:
	if {{nextest}} = "true"; then \
	  cargo nextest run; \
	else \
	  cargo test --workspace -- --test-threads=6; \
	fi
	cargo test --manifest-path tests/template/Cargo.toml

integration-setup:
	rm -rf ~/.cache/wick
	./etc/integration/postgres.sh up init
	./etc/integration/mssql.sh up init
	./etc/integration/registry.sh up init
	./etc/integration/httpbin.sh up init
	cargo run -p wick-cli -- reg push --debug ./crates/integration/test-baseline-component/component.yaml --insecure-oci=${DOCKER_REGISTRY}

integration-teardown:
	# docker rm -f simple_registry
	./etc/integration/registry.sh down
	./etc/integration/postgres.sh down
	./etc/integration/mssql.sh down
	./etc/integration/httpbin.sh down

deps:
	just dep-install-nightly
	npm install -g apex-template prettier ts-node commitlint conventional-changelog-conventionalcommits
	cargo install cargo-deny cargo-nextest

dep-install-nightly:
	#!/usr/bin/env bash
	if [[ "{{rustup}}" == "true" ]]; then
	   if [[ "$(rustup toolchain list | grep nightly)" == "" ]]; then
	     echo "Installing rust nightly..."
	     rustup toolchain install nightly;
	   else
	     echo "Rust nightly found."
	   fi
	else
	  echo "Could not find rustup, please ensure you have rust nightly installed if you intend to build from source."
	fi



update-lints:
	ts-node ./etc/update-lints.ts

lint-commits:
	npx commitlint --config ./etc/commitlint.config.js --from $(git describe --all origin --abbrev=0) --to HEAD --verbose

publish-sdk VERSION *FLAGS:
	cargo release {{VERSION}} {{FLAGS}} -p wick-packet -p wick-component-codegen -p wick-interface-types -p wick-component -p wick-config -p flow-expression-parser

unused:
	for crate in crates/bins/*; do ./etc/udeps.sh $crate; done
	for crate in crates/wick/*; do ./etc/udeps.sh $crate; done
	for crate in crates/misc/*; do ./etc/udeps.sh $crate; done