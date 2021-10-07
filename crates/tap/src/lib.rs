pub mod error;

pub use error::TestError;
use testanything::tap_test::TapTest;
use testanything::tap_test_builder::TapTestBuilder;

#[derive(Default)]
pub struct TestRunner {
  desc: Option<String>,
  blocks: Vec<TestBlock>,
  output: Vec<String>,
}

impl TestRunner {
  pub fn new<T: AsRef<str>>(desc: Option<T>) -> Self {
    Self {
      desc: desc.map(|v| v.as_ref().to_owned()),
      blocks: vec![],
      output: vec![],
    }
  }

  pub fn add_block(&mut self, block: TestBlock) {
    self.blocks.push(block);
  }

  pub fn get_tap_lines(&self) -> &Vec<String> {
    &self.output
  }

  pub fn run(&mut self) {
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
    for block in self.blocks.iter_mut() {
      if let Some(desc) = block.desc.as_ref() {
        all_lines.push(format!("# {}", desc));
      }
      let mut block_passed = true;
      for result in block.run() {
        test_num += 1;
        let tap = result.status_line(test_num);
        all_lines.push(tap);
        block_passed = block_passed && result.passed;
        if !result.passed {
          let mut formatted_diagnostics = format_diagnostics(&result.diagnostics);
          all_lines.append(&mut formatted_diagnostics);
        }
      }
      if !block_passed {
        all_lines.append(&mut format_diagnostics(&block.diagnostics));
      }
    }
    self.output = all_lines;
  }

  pub fn print(&self) {
    let lines = self.get_tap_lines();
    for line in lines {
      println!("{}", line);
    }
  }

  pub fn num_failed(&self) -> u32 {
    let lines = self.get_tap_lines();
    let mut num_failed: u32 = 0;

    for line in lines {
      if line.starts_with("not ok") {
        num_failed += 1;
      }
    }
    num_failed
  }
}

pub fn format_diagnostic_line<T: AsRef<str>>(line: T) -> String {
  format!("# {}", line.as_ref())
}

pub fn format_diagnostics<T>(lines: &[T]) -> Vec<String>
where
  T: AsRef<str>,
{
  lines.iter().map(format_diagnostic_line).collect()
}

#[derive(Default)]
pub struct TestBlock {
  desc: Option<String>,
  tests: Vec<TestCase>,
  pub diagnostics: Vec<String>,
}

impl TestBlock {
  pub fn new<T: AsRef<str>>(desc: Option<T>) -> Self {
    Self {
      desc: desc.map(|v| v.as_ref().to_owned()),
      tests: vec![],
      diagnostics: vec![],
    }
  }

  pub fn add_test<T: AsRef<str>>(
    &mut self,
    test: impl FnOnce() -> bool + 'static,
    description: T,
    diagnostics: Option<Vec<String>>,
  ) {
    self.tests.push(TestCase {
      test: Some(Box::new(test)),
      result: Some(false),
      description: description.as_ref().to_owned(),
      diagnostics: diagnostics.unwrap_or_else(Vec::new),
    });
  }

  pub fn num_tests(&self) -> usize {
    self.tests.len()
  }

  pub fn run(&mut self) -> Vec<TapTest> {
    let mut tests = vec![];
    for test_case in self.tests.iter_mut() {
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
  test: Option<Box<dyn FnOnce() -> bool>>,
  result: Option<bool>,
  description: String,
  diagnostics: Vec<String>,
}

impl TestCase {
  pub fn exec(&mut self) -> bool {
    match self.test.take() {
      Some(test) => {
        let result = (test)();
        self.result = Some(result);
        result
      }
      None => self
        .result
        .expect("Attempted to execute a test without a test case"),
    }
  }
}
