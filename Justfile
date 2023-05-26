
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
  @echo "Use `just install-debug` to build debug version."
  cargo install --path crates/bins/wick

# Build wick binary with debug symbols and additional output
install-debug:
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
  just wasm-clean

# Run the basic suite of tests
test: codegen early-errors wasm unit-tests

# Run unit tests
unit-tests: _codegen-tests
  if {{nextest}} = "true"; then \
    cargo nextest run -E 'not (test(slow_test) | test(integration_test))'; \
  else \
    cargo test --workspace -- --skip integration_test --skip slow_test --skip codegen-test --test-threads=6; \
  fi

# Run integration tests
integration-tests: _codegen-tests
  if {{nextest}} = "true"; then \
    cargo nextest run -E 'not (test(slow_test))'; \
  else \
    cargo test --workspace -- --skip slow_test --test-threads=6; \
  fi
  cargo test --manifest-path tests/template/Cargo.toml
  just _wick-db-tests

# Run tests suitable for CI
ci-tests: wasm
  just unit-tests

# Run unit, integration, and slow tests
all-tests:
  if {{nextest}} = "true"; then \
    cargo nextest run; \
  else \
    cargo test --workspace -- --test-threads=6; \
  fi
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
  # docker rm -f simple_registry
  just _run-integration-task down

# Run codegen-related tasks
codegen:
  just crates/wick/wick-config/codegen

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

##################################
### Private, dependency tasks. ###
##################################

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
    print("Running {}".format([bin] + sys.argv[1:]))
    # subprocess.call([bin] + sys.argv[1:])

_run-wasm-task task:
  #!{{python}}
  import subprocess
  wasm = [
    "crates/integration/test-baseline-component",
    "crates/integration/test-http-trigger-component",
    "crates/integration/test-cli-trigger-component",
    "crates/integration/test-cli-with-db",
    "examples/http/middleware/request"
  ]
  for dir in wasm:
    subprocess.call(["just", "{}/{{task}}".format(dir)])

# Run `wick` tests for db components
_wick-db-tests:
  cargo run -p wick-cli -- test ./examples/db/flow-with-postgres.wick
  cargo run -p wick-cli -- test ./examples/db/postgres-numeric.wick


# Run component-codegen unit tests
_codegen-tests:
  just tests/codegen-tests/codegen
  just tests/codegen-tests/test
