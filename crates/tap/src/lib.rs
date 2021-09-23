pub mod error;

pub use error::TestError;
use testanything::tap_test::TapTest;
use testanything::tap_test_builder::TapTestBuilder;

#[derive(Default)]
pub struct TestRunner {
  desc: Option<String>,
  blocks: Vec<TestBlock>,
}

impl TestRunner {
  pub fn new<T: AsRef<str>>(desc: Option<T>) -> Self {
    Self {
      desc: desc.map(|v| v.as_ref().to_owned()),
      blocks: vec![],
    }
  }

  pub fn add_block(&mut self, block: TestBlock) {
    self.blocks.push(block);
  }

  pub fn get_tap_lines(self) -> Vec<String> {
    let description = self
      .desc
      .as_ref()
      .map_or_else(|| "TAP Stream".to_owned(), |v| v.to_owned());

    let mut total_tests = 0;
    for block in &self.blocks {
      total_tests += block.num_tests();
    }

    let name = format!("# {}", description);
    let plan_line = format!("1..{}", total_tests);
    let mut all_lines = vec![name, plan_line];

    let mut test_num = 0;
    for block in self.blocks.into_iter() {
      if let Some(desc) = block.desc.as_ref() {
        all_lines.push(format!("# {}", desc));
      }
      for result in block.run() {
        test_num += 1;
        let tap = result.status_line(test_num);
        all_lines.push(tap);
        if !result.passed {
          let mut formatted_diagnostics = result
            .diagnostics
            .iter()
            .map(|comment| result.format_diagnostics(comment))
            .collect::<Vec<String>>();
          all_lines.append(&mut formatted_diagnostics);
        }
      }
    }

    // for (i, test) in all_tests.iter().enumerate() {
    //   let index = i as i64; // by default i is a usize.
    //   let tap = test.status_line(index + 1); // TAP tests can't start with zero
    //   all_lines.push(tap);
    //   if !test.passed {
    //     let mut formatted_diagnostics = test
    //       .diagnostics
    //       .iter()
    //       .map(|comment| test.format_diagnostics(comment))
    //       .collect::<Vec<String>>();
    //     all_lines.append(&mut formatted_diagnostics);
    //   }
    // }

    all_lines
  }

  pub fn print(self) {
    let lines = self.get_tap_lines();
    for line in lines {
      println!("{}", line);
    }
  }
}

#[derive(Default)]
pub struct TestBlock {
  desc: Option<String>,
  tests: Vec<TestCase>,
}

impl TestBlock {
  pub fn new<T: AsRef<str>>(desc: Option<T>) -> Self {
    Self {
      desc: desc.map(|v| v.as_ref().to_owned()),
      tests: vec![],
    }
  }

  pub fn add_test<T: AsRef<str>>(
    &mut self,
    test: impl FnOnce() -> bool + 'static,
    description: T,
    diagnostics: Option<Vec<String>>,
  ) {
    self.tests.push(TestCase {
      test: Box::new(test),
      description: description.as_ref().to_owned(),
      diagnostics: diagnostics.unwrap_or_else(Vec::new),
    });
  }

  pub fn num_tests(&self) -> usize {
    self.tests.len()
  }

  pub fn run(self) -> Vec<TapTest> {
    let mut tests = vec![];
    for test_case in self.tests.into_iter() {
      let diags: Vec<&str> = test_case.diagnostics.iter().map(|s| s.as_str()).collect();
      let tap_test = TapTestBuilder::new()
        .name(test_case.description.clone())
        .diagnostics(&diags)
        .passed(test_case.exec())
        .finalize();
      tests.push(tap_test);
    }
    tests
  }
}

struct TestCase {
  test: Box<dyn FnOnce() -> bool>,
  description: String,
  diagnostics: Vec<String>,
}

impl TestCase {
  pub fn exec(self) -> bool {
    (self.test)()
  }
}
