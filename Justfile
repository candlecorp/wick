
wasm:
	just crates/integration/test-baseline-component/build

clean:
	cargo clean
	rm -rf node_modules
	just crates/wasmflow/wasmflow-manifest/clean

codegen:
	just crates/wasmflow/wasmflow-manifest/codegen

test: codegen early-errors wasm unit-tests

install:
	cargo build --workspace
	mv build/local/* ~/.cargo/bin/

early-errors:
	cargo +nightly fmt --check
	cargo clippy --workspace --bins
	cargo deny check licenses --config etc/deny.toml --hide-inclusion-graph

unit-tests:
	cargo build -p wasmflow
	cargo test --workspace -- --skip integration_test

integration-tests:
	cargo build -p wasmflow
	NATS_URL=$(NATS_URL) cargo test --workspace

deps:
	npm install -g apex-template prettier ts-node
	cargo install cargo-deny tomlq

update-lints:
  ts-node ./etc/update-lints.ts