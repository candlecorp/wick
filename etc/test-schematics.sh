#!/bin/bash

set -e
export RUST_LOG=wasmcloud_nats_kvcache=debug,polling=info,async_io=info,hyper=info,trace,cranelift_codegen=info,cranelift_wasm=info,wasmcloud_host::control_interface::ctlactor=debug,wasmcloud_host::capability::native_host=debug,wasmtime=info,want=info,tracing=info,wasmcloud_host::messagebus::latticecache_client=info,wasmcloud_host::messagebus::rpc_subscription=info,wasmcloud_host::messagebus::nats_subscriber=info

CMD="cargo run -q --"

TX_ID=$($CMD ctl push test str2bytes '{"input":"this is a test string as input"}' --output json | jq -r '.[0].tx_id')

echo "Transaction ID of md5 input: $TX_ID"

if [[ "$TX_ID" == "" ]]; then
  echo "NOT OK"
  exit 1
fi

sleep 1

MD5=$($CMD ctl take test md5 output --tx_id $TX_ID --output json --encoder messagepack | jq -r ".response")
echo "MD5: $MD5"

expected="edf4610a4844890420fe86901283a403"

if [[ "$MD5" == "$expected" ]]; then
  echo "OK"
else
  echo "NOT OK"
  echo "Expected:"
  echo $expected
  echo "Actual:"
  echo $MD5
  exit 1
fi
