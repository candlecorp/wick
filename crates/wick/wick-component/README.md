# wick-component SDK

This crate provides the `wick-component` SDK used to build components for the Wick runtime. It is the primary dependency used by the `wick-component-codegen` generator and re-exports common dependencies for Wick components.


## Re-exported dependencies

These are exposed at the root of the crate, i.e. `wick_component::packet` or `wick_component::wasmrs`.

- `wick-packet` as `packet`
- `flow-component`
- `wasmrs`
- `wasmrs-guest`
- `wasmrs-runtime`
- `wasmrs-codec`
- `wasmrs-rx`
- `bytes` with feature `bytes`
- `chrono` with feature `datetime`
- Partial export of `serde-json`
- Partial export of `tokio-stream`

See docs.rs for exact details.

## Macros

- `propagate_if_error!`

If the passed result is an error, the error propagates to all downstream outputs of a wick component and breaks execution with the provided statement. If the passed result is an `Ok`, the unwrapped result is returned.


