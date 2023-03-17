
wasm:
	just crates/integration/test-baseline-component/build
	just crates/integration/test-http-trigger-component/build
	just crates/integration/test-cli-trigger-component/build

debug-wasm:
	just crates/integration/test-baseline-component/debug
	just crates/integration/test-http-trigger-component/debug
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
	npm install -g apex-template prettier ts-node commitlint conventional-changelog-conventionalcommits
	cargo install cargo-deny

update-lints:
  ts-node ./etc/update-lints.ts

lint-commits:
  npx commitlint --config ./etc/commitlint.config.js --from $(git describe --all origin --abbrev=0) --to HEAD --verbose

publish-sdk VERSION="minor":
  cargo release {{VERSION}} -p wick-packet -p wick-component-codegen -p wick-interface-types -p wick-component -p seeded-random -p wick-config -p flow-expression-parser
