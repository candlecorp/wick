#![doc = include_str!("../README.md")]
// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::await_holding_lock,
  clippy::borrow_as_ptr,
  clippy::branches_sharing_code,
  clippy::cast_lossless,
  clippy::clippy::collection_is_never_read,
  clippy::cloned_instead_of_copied,
  clippy::cognitive_complexity,
  clippy::create_dir,
  clippy::deref_by_slicing,
  clippy::derivable_impls,
  clippy::derive_partial_eq_without_eq,
  clippy::equatable_if_let,
  clippy::exhaustive_structs,
  clippy::expect_used,
  clippy::expl_impl_clone_on_copy,
  clippy::explicit_deref_methods,
  clippy::explicit_into_iter_loop,
  clippy::explicit_iter_loop,
  clippy::filetype_is_file,
  clippy::flat_map_option,
  clippy::format_push_string,
  clippy::fn_params_excessive_bools,
  clippy::future_not_send,
  clippy::get_unwrap,
  clippy::implicit_clone,
  clippy::if_then_some_else_none,
  clippy::impl_trait_in_params,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::inherent_to_string,
  clippy::iter_not_returning_iterator,
  clippy::large_types_passed_by_value,
  clippy::large_include_file,
  clippy::let_and_return,
  clippy::manual_assert,
  clippy::manual_ok_or,
  clippy::manual_split_once,
  clippy::manual_let_else,
  clippy::manual_string_new,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::missing_enforced_import_renames,
  clippy::missing_assert_message,
  clippy::missing_const_for_fn,
  clippy::must_use_candidate,
  clippy::mut_mut,
  clippy::needless_for_each,
  clippy::needless_option_as_deref,
  clippy::needless_pass_by_value,
  clippy::needless_collect,
  clippy::needless_continue,
  clippy::non_send_fields_in_send_ty,
  clippy::nonstandard_macro_braces,
  clippy::option_if_let_else,
  clippy::option_option,
  clippy::rc_mutex,
  clippy::redundant_else,
  clippy::same_name_method,
  clippy::semicolon_if_nothing_returned,
  clippy::str_to_string,
  clippy::string_to_string,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::trivial_regex,
  clippy::try_err,
  clippy::unnested_or_patterns,
  clippy::unused_async,
  clippy::unwrap_or_else_default,
  clippy::useless_let_if_seq,
  bad_style,
  clashing_extern_declarations,
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
#![warn(clippy::exhaustive_enums)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow()]

use testanything::tap_test::TapTest;
use testanything::tap_test_builder::TapTestBuilder;

#[derive(Default, Debug)]
/// [TestRunner] is the main way you'll interact with TAP tests.
pub struct TestRunner {
  desc: Option<String>,
  blocks: Vec<TestBlock>,
  output: Vec<String>,
}

impl TestRunner {
  /// Create a new [TestRunner]
  #[must_use]
  pub const fn new(desc: Option<String>) -> Self {
    Self {
      desc,
      blocks: vec![],
      output: vec![],
    }
  }

  /// Add a [TestBlock] to the runner.
  pub fn add_block(&mut self, block: TestBlock) {
    self.blocks.push(block);
  }

  #[must_use]
  /// Get the TAP output.
  pub const fn get_tap_lines(&self) -> &Vec<String> {
    &self.output
  }

  /// Execute the tests.
  pub fn run(&mut self) {
    let description = self
      .desc
      .as_ref()
      .map_or_else(|| "TAP Stream".to_owned(), |v| v.clone());

    let mut total_tests = 0;
    for block in &self.blocks {
      total_tests += block.num_tests();
    }

    let plan_line = format!("1..{} # {}", total_tests, description);
    let mut all_lines = vec![plan_line];

    let mut test_num = 0;
    for block in &mut self.blocks {
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

  /// Print the TAP output.
  pub fn print(&self) {
    let lines = self.get_tap_lines();
    for line in lines {
      println!("{}", line);
    }
  }

  #[must_use]
  /// Get the number of failed tests.
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

fn format_diagnostic_line<T: std::fmt::Display>(line: T) -> String {
  format!("# {}", line)
}

fn format_diagnostics<T>(lines: &[T]) -> Vec<String>
where
  T: std::fmt::Display,
{
  lines.iter().map(format_diagnostic_line).collect()
}

#[derive(Default, Debug)]
/// A [TestBlock] organizes test cases together under one umbrella.
pub struct TestBlock {
  desc: Option<String>,
  tests: Vec<TestCase>,
  diagnostics: Vec<String>,
}

impl TestBlock {
  /// Create a new [TestBlock].
  #[must_use]
  pub const fn new(desc: Option<String>) -> Self {
    Self {
      desc,
      tests: vec![],
      diagnostics: vec![],
    }
  }

  /// Add a new test case.
  pub fn add_test<T: Into<String>>(
    &mut self,
    test: impl FnOnce() -> bool + Sync + Send + 'static,
    description: T,
    diagnostics: Option<Vec<String>>,
  ) {
    self.tests.push(TestCase {
      test: Some(Box::new(test)),
      result: Some(false),
      description: description.into(),
      diagnostics,
    });
  }

  /// Add a test failure.
  pub fn fail<T: Into<String>>(&mut self, description: T, diagnostics: Option<Vec<String>>) {
    self.tests.push(TestCase {
      test: None,
      result: Some(false),
      description: description.into(),
      diagnostics,
    });
  }

  /// Add a test success.
  pub fn succeed<T: Into<String>>(&mut self, description: T, diagnostics: Option<Vec<String>>) {
    self.tests.push(TestCase {
      test: None,
      result: Some(true),
      description: description.into(),
      diagnostics,
    });
  }

  /// Add diagnostic messages to this test block.
  pub fn add_diagnostic_messages(&mut self, messages: Vec<String>) {
    self.diagnostics = messages;
  }

  fn num_tests(&self) -> usize {
    self.tests.len()
  }

  /// Execute the [TestBlock]'s test cases.
  pub fn run(&mut self) -> Vec<TapTest> {
    let mut tests: Vec<TapTest> = vec![];
    for test_case in &mut self.tests {
      let mut tap_test = TapTestBuilder::new();
      tap_test.name(test_case.description.clone());

      if let Some(diag) = &test_case.diagnostics {
        let diags: Vec<&str> = diag.iter().map(|s| s.as_str()).collect();
        tap_test.diagnostics(&diags);
      }

      let tap_test = tap_test.passed(test_case.exec()).finalize();
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
  diagnostics: Option<Vec<String>>,
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
      None => self.result.unwrap_or(false),
    }
  }
}
