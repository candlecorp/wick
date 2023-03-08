
wasm:
	just crates/integration/test-baseline-component/build

clean:
	cargo clean
	rm -rf node_modules
	just crates/wick/wick-config-component/clean

codegen:
	just crates/wick/wick-config-component/codegen

test: codegen early-errors wasm unit-tests

install-release:
	cargo install --path crates/bins/wick --release

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

integration-tests:
	cargo build -p wick
	NATS_URL=$(NATS_URL) cargo test --workspace

deps:
	npm install -g apex-template prettier ts-node
	cargo install cargo-deny tomlq

update-lints:
  ts-node ./etc/update-lints.ts