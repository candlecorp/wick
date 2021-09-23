use tap::{
  TestBlock,
  TestRunner,
};

#[test]
fn basics() -> Result<(), tap::TestError> {
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
  let expected = vec![
    "# My test",
    "1..2",
    "# My block",
    "ok 1 three is greater than two",
    "not ok 2 three is less than two",
    "# three was not less than two",
  ];
  let lines = runner.get_tap_lines();
  for (i, line) in lines.iter().enumerate() {
    println!("{}", line);
    assert_eq!(line, expected[i]);
  }

  Ok(())
}
