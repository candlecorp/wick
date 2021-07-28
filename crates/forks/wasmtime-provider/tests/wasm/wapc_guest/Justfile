all:
  just interface
  just wasm

interface:
  wapc generate wapc.yaml

wasm:
  cargo build --target=wasm32-unknown-unknown
  cp target/wasm32-unknown-unknown/debug/simple.wasm test.wasm