# wick-operation

wick-operation is a crate that provides the `operation` proc macro exposed by the `wick-component` crate.

## Usage

```rust

use wick_component::operation;

#[operation(unary_simple)]
fn my_operation(my_input: String) -> anyhow::Result<String> {
    // ...
}
```
