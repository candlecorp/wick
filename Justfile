
wasm:
	just crates/integration/test-baseline-component/build
	just crates/integration/test-cli-trigger-component/build

debug-wasm:
	just crates/integration/test-baseline-component/debug
	just crates/integration/test-cli-trigger-component/debug

clean:
	cargo clean
	rm -rf node_modules
	just crates/wick/wick-config/clean

codegen:
	just crates/wick/wick-config/codegen

test: codegen early-errors wasm unit-tests

install-debug:
	cargo install --path crates/bins/wick --profile=dev

install:
	cargo install --path crates/bins/wick

early-errors: licenses
	cargo +nightly fmt --check
	cargo clippy --workspace --bins

licenses:
	cargo deny --workspace check licenses  --config etc/deny.toml --hide-inclusion-graph

unit-tests:
	cargo build -p wick
	cargo test --workspace -- --skip integration_test

ci-tests: wasm
  just unit-tests

integration-tests:
	cargo build -p wick
	NATS_URL=$(NATS_URL) cargo test --workspace

deps:
	npm install -g apex-template prettier ts-node
	cargo install cargo-deny

update-lints:
  ts-node ./etc/update-lints.ts

publish-sdk VERSION="minor":
  cargo release {{VERSION}} -p wick-packet wick-codegen wick-interface-types wick-component
