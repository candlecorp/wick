
# Variable that stores if `cargo-nextest` was found
nextest := `command -v cargo-nextest >/dev/null && echo true || echo false`

# Variable that stores if `rustup` was found
rustup := `command -v rustup >/dev/null && echo true || echo false`

# use #!{{python}} to ensure that the python path is correct across platforms.
python := if os() == 'windows' { 'python' } else { '/usr/bin/env python3' }

set dotenv-load
set positional-arguments
set export

# Print help message
default:
  @just --list --unsorted

# Build optimized wick binary
install:
  @echo "Building release version of wick..."
  @echo "Use 'just install-debug' to build debug version."
  cargo install --path crates/bins/wick

# Build an optimized wick binary with debug symbols
install-debug:
  cargo install --profile=release-with-debug --path crates/bins/wick

# Build wick binary with debug symbols and additional output
install-dev:
  cargo install --path crates/bins/wick --debug

# Build wasm binaries that wick tests depend on
wasm:
  just _run-wasm-task build

# Build unoptimized wasm binaries for additional debug output.
wasm-debug:
  just _run-wasm-task debug

# Run "clean" task in wasm projects.
wasm-clean:
  just _run-wasm-task clean

# Remove build artifacts & generated code.
clean:
  cargo clean
  rm -rf node_modules
  just crates/wick/wick-config/clean
  just crates/wick/wick-rpc/clean
  just wasm-clean

# Run the basic suite of tests
test: codegen early-errors wasm unit-tests

# Run unit tests
unit-tests: _check_nextest _codegen-tests
  cargo nextest run -E 'not (test(slow_test) | test(integration_test))'

# Run integration tests
integration-tests: _check_nextest _codegen-tests
  cargo nextest run -E 'not (test(slow_test))'
  just wick-tests
  cargo test --manifest-path tests/template/Cargo.toml

# Tests run via `wick test`
wick-tests:
  just _wick-component-tests
  just _wick-http-tests
  just _wick-db-tests

# Run tests suitable for CI
ci-tests: wasm
  just unit-tests

# Run unit, integration, and slow tests
all-tests: _check_nextest
  cargo nextest run
  cargo test --manifest-path tests/template/Cargo.toml

# Run integration setup, tests, and teardown in one task
integration: integration-setup && integration-teardown
  just integration-tests

# Set up the environment for integration tests
integration-setup:
  rm -rf ~/.cache/wick
  just _run-integration-task up init
  cargo run -p wick-cli -- reg push --debug ./crates/integration/test-baseline-component/component.yaml --insecure-oci=${DOCKER_REGISTRY}

# Tear down the environment for integration tests
integration-teardown:
  just _run-integration-task down

# Run codegen-related tasks
codegen:
  just crates/wick/wick-rpc/codegen
  just crates/wick/wick-config/codegen
  cp crates/wick/wick-config/docs/v*.md docs/content/configuration/reference/

# Run lints, license checks, formatting checks, et al.
early-errors: licenses
  cargo +nightly fmt --check
  cargo clippy --workspace --bins

# Check dependency licences
licenses:
  cargo deny --workspace check licenses  --config etc/deny.toml --hide-inclusion-graph

# Install dependencies necessary to build & test wick
deps:
  just _dep-install-nightly
  npm install -g apex-template prettier ts-node commitlint conventional-changelog-conventionalcommits
  cargo install cargo-deny cargo-nextest

# Check the commits since origin to ensure they follow the conventional commit format
lint-commits:
  npx commitlint --config ./etc/commitlint.config.js --from $(git describe --all origin --abbrev=0) --to HEAD --verbose

# Update the lint configuration in each rust crate
update-lints:
  ts-node ./etc/update-lints.ts

# Run a basic test for unused dependencies in rust crates.
check-unused:
  for crate in crates/bins/*; do ./etc/udeps.sh $crate; done
  for crate in crates/wick/*; do ./etc/udeps.sh $crate; done
  for crate in crates/misc/*; do ./etc/udeps.sh $crate; done

# Developer task to run a quick subset of tasks and tests.
sanity: early-errors
  just unit-tests

# Build the document site
docsite:
  cd docs && hugo --minify

# Run the development hugo server
devdocs:
  cd docs && hugo serve --disableFastRender --cleanDestinationDir --ignoreCache --gc

# Run `cargo doc` to generate rust documentation and copy it to the docs site
rustdoc:
  RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo +nightly doc --workspace --no-deps
  mkdir -p docs/static/rustdoc/
  rsync -av --delete --exclude=".*" target/doc/ docs/static/rustdoc/

##################################
### Private, dependency tasks. ###
##################################

_check_nextest:
  #!{{python}}
  if "{{nextest}}" != "true":
    print ("Tests use cargo-nextest, please install it with: cargo install cargo-nextest")
    exit(1)

_dep-install-nightly:
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

_run-integration-task *args='':
  #!{{python}}
  import subprocess
  import sys
  bins = [
    "./etc/integration/postgres.sh",
    "./etc/integration/mssql.sh",
    "./etc/integration/registry.sh",
    "./etc/integration/httpbin.sh"
  ]
  for bin in bins:
    if subprocess.call([bin] + sys.argv[1:]) != 0:
      exit(1)

_run-wasm-task task:
  #!{{python}}
  import subprocess
  wasm = [
    "crates/integration/test-baseline-component",
    "crates/integration/test-http-trigger-component",
    "crates/integration/test-cli-trigger-component",
    "crates/integration/test-cli-with-db",
    "examples/http/middleware/request",
    "examples/http/wasm-http-call/wasm-component",
  ]
  for dir in wasm:
    if subprocess.call(["just", "{}/{{task}}".format(dir)]) != 0:
      exit(1)

# Run `wick` tests for db components
_wick-db-tests:
  cargo run -p wick-cli -- test ./examples/db/postgres-numeric-tests.wick

# Run `wick` tests for http components
_wick-http-tests:
  cargo run -p wick-cli -- test ./examples/http/wasm-http-call/harness.wick

# Run `wick` tests for generic components
_wick-component-tests:
  cargo run -p wick-cli -- test ./examples/components/hello-world.wick
  cargo run -p wick-cli -- test ./examples/components/composite-db-import.wick

# Run component-codegen unit tests
_codegen-tests:
  just tests/codegen-tests/codegen
  just tests/codegen-tests/test
