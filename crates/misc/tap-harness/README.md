# tap-harness

This library is a wrapper to write tests that generate output in the Test Anything Protocol (TAP) format.

## Usage

```rust
use tap_harness::{TestBlock, TestRunner};

fn main() -> anyhow::Result<()> {
  let mut runner = TestRunner::new(Some("My test"));

  let mut block = TestBlock::new(Some("My block"));

  block.add_test(
    || 3 > 2,
    "three is greater than two",
    Some(vec!["three was not greater".to_owned()]),
  );

  block.add_test(
    || 3 < 2,
    "three is less than two",
    Some(vec!["three was not less than two".to_owned()]),
  );

  runner.add_block(block);

  runner.run();

  let lines = runner.get_tap_lines();

  // or

  runner.print();

  Ok(())
}
```

This prints:

```console
# My test
1..2
# My block
ok 1 three is greater than two
not ok 2 three is less than two
# three was not less than two
```

## Additional links

- [TAP homepage](https://testanything.org)
- [List of tools that read TAP output](https://testanything.org/consumers.html)
- [Test_Anything_Protocol on Wikipedia](https://en.wikipedia.org/wiki/Test_Anything_Protocol)
