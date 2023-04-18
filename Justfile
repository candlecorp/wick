
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

unit-tests:
	cargo test --workspace -- --skip integration_test --test-threads=6

ci-tests: wasm
  just unit-tests

integration: integration-setup && integration-teardown
	just integration-tests

integration-tests:
	cargo test --workspace

integration-setup:
	rm -rf ~/.cache/wick
	./etc/integration/postgres.sh up init
	./etc/integration/mssql.sh up init
	./etc/integration/registry.sh up init
	cargo run -p wick-cli -- reg push --debug ${DOCKER_REGISTRY}/test-component/baseline:0.1.0 ./crates/integration/test-baseline-component/component.yaml --insecure=${DOCKER_REGISTRY}

integration-teardown:
	# docker rm -f simple_registry
	./etc/integration/registry.sh down
	./etc/integration/postgres.sh down
	./etc/integration/mssql.sh down

deps:
	npm install -g apex-template prettier ts-node commitlint conventional-changelog-conventionalcommits
	cargo install cargo-deny

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