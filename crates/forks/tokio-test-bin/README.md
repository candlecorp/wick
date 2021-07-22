# test_bin

A crate for getting the crate binary in an integration test.

If you are writing a command-line interface app then it is useful to write an integration test that uses the binary. You most likely want to launch the binary and inspect the output. This module lets you get the binary so it can be tested.

## Example

Here is the basic usage:

```rust
let output = test_bin::get_test_bin("my_cli_app")
    .output()
    .expect("Failed to start my_binary");
assert_eq!(
    String::from_utf8_lossy(&output.stdout),
    "Output from my CLI app!\n"
);
```

## Acknowledgements

The `cargo` and `ripgrep` crates were used as inspiration. They both test their
binaries using a similar approach. The `cargo` crate's documentation and license
was used as a starting point.

## Contributing

See CONTRIBUTING.md.

## License

The `test_bin` crate is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT for details.
