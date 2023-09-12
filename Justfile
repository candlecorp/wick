
# Variable that stores if `cargo-nextest` was found
nextest := `command -v cargo-nextest >/dev/null && echo true || echo false`

# Variable that stores if `rustup` was found
rustup := `command -v rustup >/dev/null && echo true || echo false`

# use #!{{python}} to ensure that the python path is correct across platforms.
python := if os() == 'windows' { 'python' } else { '/usr/bin/env python3' }

# The wick repository
repository := "https://github.com/candlecorp/wick"

# The `wick` command to ensure that the build from source is used.
wick := "cargo run -p wick-cli --"

# The root directory of this project
wick_root := justfile_directory()

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
  cargo install --path .

# Build an optimized wick binary with debug symbols
install-debug:
  cargo install --profile=release-with-debug --path .

# Build wick binary with debug symbols and additional output
install-dev:
  cargo install --path . --debug

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
  cargo nextest run --workspace -E 'not (test(slow_test) | test(integration_test))'
  cargo nextest run -p cli-tests -E 'not (test(slow_test) | test(integration_test))'
  just wick-unit-tests

# Run integration tests
integration-tests: _check_nextest _codegen-tests
  cargo nextest run --workspace -E 'not (test(slow_test))'
  cargo nextest run -p cli-tests -E 'not (test(slow_test))'
  just wick-unit-tests
  just wick-integration-tests
  cargo test --manifest-path integration-tests/template/Cargo.toml

# Tests run via `wick test`
wick-unit-tests:
  just _wick-component-tests

# Integration tests run via `wick test`
wick-integration-tests:
  just _wick-http-tests
  just _wick-db-tests

# Run tests suitable for CI
ci-tests: install-dev wasm
  just unit-tests

# Run unit, integration, and slow tests
all-tests: _check_nextest
  cargo nextest run --workspace
  cargo test --manifest-path integration-tests/template/Cargo.toml

# Run integration setup, tests, and teardown in one task
integration: integration-setup && integration-teardown
  just integration-tests

# Set up the environment for integration tests
integration-setup:
  rm -rf ~/.cache/wick
  just _run-integration-task up init
  {{wick}} reg push --debug ./crates/integration/test-baseline-component/component.yaml --insecure-oci=${DOCKER_REGISTRY}

# Tear down the environment for integration tests
integration-teardown:
  just _run-integration-task down

# Run codegen-related tasks
codegen:
  just crates/wick/wick-rpc/codegen
  just crates/wick/wick-config/codegen
  cp crates/wick/wick-config/docs/v*.md docs/content/wick/configuration/reference/

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

# Quickly generate a new rust WebAssembly example
new-rust-example name path="examples/components":
  echo "Generating new rust example: {{name}}"
  echo "Note: This is optimized for macOS, if it breaks on your platform, please open an issue at {{repository}}"
  cd {{path}} && cargo generate -p {{wick_root}}/templates/rust --name {{name}}

  RELATIVE=$(realpath --relative-to={{wick_root}}/{{path}}/{{name}} {{wick_root}}); \
    sed -E -i '' "s,([A-Za-z0-9_-]*) = { git = \\\"{{repository}}.git\\\",\\1 = { path = \\\"$RELATIVE/crates/wick/\\1\\\",g" "{{wick_root}}/{{path}}/{{name}}/Cargo.toml"; \
    sed -E -i '' "s,wick := \\\"wick\\\",wick := \\\"cargo run --manifest-path=$RELATIVE/Cargo.toml -p wick-cli --\\\",g" "{{wick_root}}/{{path}}/{{name}}/justfile";

  echo "New rust example generated at {{path}}/{{name}}"

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
    "examples/components/wasi-fs",
    "examples/components/config-generator",
    "examples/components/cli-trigger",
    "examples/cli/wasm-cli",
    "examples/http/wasm-http-call/wasm-component",
  ]
  for dir in wasm:
    if subprocess.call(["just", "{}/{{task}}".format(dir)]) != 0:
      exit(1)

# Run `wick` tests for db components
_wick-db-tests:
  {{wick}} test ./examples/db/tests/postgres-numeric-tests.wick
  {{wick}} test ./examples/db/tests/postgres-null-tests.wick
  {{wick}} test ./examples/db/tests/postgres-date-tests.wick
  {{wick}} test ./examples/db/postgres-component.wick
  {{wick}} test ./examples/db/azuresql-component.wick
  {{wick}} test ./examples/db/sqlite-component.wick
  {{wick}} test ./examples/db/sqlite-inmemory-component.wick
  {{wick}} test ./integration-tests/cli-tests/tests/cmd/db/azuresql-tx-test.wick
  {{wick}} test ./examples/components/composite-db-import.wick

# Run `wick` tests for http components
_wick-http-tests:
  {{wick}} test ./examples/http/http-client.wick
  {{wick}} test ./examples/http/wasm-http-call/harness.wick

# Run `wick` tests for generic components
_wick-component-tests:
  {{wick}} test ./examples/components/hello-world.wick
  {{wick}} test ./examples/components/wasi-fs/component.wick
  {{wick}} test ./examples/components/composite-imports.wick
  {{wick}} test ./examples/components/composite-provides.wick
  {{wick}} test ./examples/components/tests.wick

# Run component-codegen unit tests
_codegen-tests:
  just integration-tests/codegen-tests/codegen
  just integration-tests/codegen-tests/test
