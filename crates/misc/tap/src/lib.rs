// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  // next version, see: https://github.com/rust-lang/rust-clippy/blob/master/CHANGELOG.md
  // clippy::manual_split_once,
  // clippy::derivable_impls,
  // clippy::needless_option_as_deref,
  // clippy::iter_not_returning_iterator,
  // clippy::same_name_method,
  // clippy::manual_assert,
  // clippy::non_send_fields_in_send_ty,
  // clippy::equatable_if_let,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

pub mod error;

pub use error::TestError;
use testanything::tap_test::TapTest;
use testanything::tap_test_builder::TapTestBuilder;

#[derive(Default, Debug)]
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

  #[must_use]
  pub fn get_tap_lines(&self) -> &Vec<String> {
    &self.output
  }

  pub fn run(&mut self) {
    let description = self
      .desc
      .as_ref()
      .map_or_else(|| "TAP Stream".to_owned(), |v| v.clone());

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

  #[must_use]
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

#[derive(Default, Debug)]
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
    test: impl FnOnce() -> bool + Sync + Send + 'static,
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

  fn num_tests(&self) -> usize {
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

#[derive()]
struct TestCase {
  test: Option<Box<dyn FnOnce() -> bool + Sync + Send>>,
  result: Option<bool>,
  description: String,
  diagnostics: Vec<String>,
}

impl std::fmt::Debug for TestCase {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TestCase")
      .field("result", &self.result)
      .field("description", &self.description)
      .field("diagnostics", &self.diagnostics)
      .finish()
  }
}

impl TestCase {
  fn exec(&mut self) -> bool {
    match self.test.take() {
      Some(test) => {
        let result = (test)();
        self.result = Some(result);
        result
      }
      #[allow(clippy::expect_used)]
      None => self
        .result
        .expect("Attempted to execute a test without a test case"),
    }
  }
}
