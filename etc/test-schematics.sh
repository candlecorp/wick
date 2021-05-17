#!/bin/bash

set -e
export RUST_LOG=wasmcloud_nats_kvcache=debug,polling=info,async_io=info,hyper=info,trace,cranelift_codegen=info,cranelift_wasm=info,wasmcloud_host::control_interface::ctlactor=debug,wasmcloud_host::capability::native_host=debug,wasmtime=info,want=info,tracing=info,wasmcloud_host::messagebus::latticecache_client=info,wasmcloud_host::messagebus::rpc_subscription=info,wasmcloud_host::messagebus::nats_subscriber=info

echo "Building..."

cargo build

CMD="cargo run -q --"

echo "Running command"

JSON=$($CMD ctl request test '{"input_data1":"this is a test string as input"}' --output json --encoder messagepack)

echo "JSON Output: $JSON"

echo "Parsing with jq"

OUTPUT=$(echo $JSON | jq -r ".response.output_data1")

echo "Value grabbed from JSON: $OUTPUT"

expected="edf4610a4844890420fe86901283a403"

if [[ "$OUTPUT" == "$expected" ]]; then
  echo "OK"
else
  echo "NOT OK"
  echo "Expected:"
  echo $expected
  echo "Actual:"
  echo $OUTPUT
  exit 1
fi
