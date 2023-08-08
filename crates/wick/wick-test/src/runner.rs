use std::time::Duration;

use flow_component::{panic_callback, SharedComponent};
use tap_harness::{TestBlock, TestRunner};
use tokio_stream::StreamExt;
use wick_interface_types::{Field, OperationSignature};
use wick_packet::{Entity, Invocation, RuntimeConfig};

use crate::utils::{gen_packet, render_config};
use crate::{get_payload, TestError, UnitTest};

#[must_use]
pub fn get_description(test: &UnitTest) -> String {
  format!(
    "(test name='{}', operation='{}')",
    test.test.name().map_or("Test", |v| v.as_str()),
    test.test.operation()
  )
}

pub async fn run_test<'a, 'b>(
  name: impl AsRef<str> + Sync + Send,
  defs: Vec<&'a mut UnitTest<'a>>,
  id: Option<&'b str>,
  component: SharedComponent,
  root_config: Option<RuntimeConfig>,
) -> Result<TestRunner, TestError> {
  let mut harness = TestRunner::new(Some(name));

  for (i, def) in defs.into_iter().enumerate() {
    let entity = id.map_or_else(
      || Entity::local(def.test.operation()),
      |id| Entity::operation(id, def.test.operation()),
    );
    let block = run_unit(i, def, entity, component.clone(), root_config.clone()).await?;
    harness.add_block(block);
  }

  harness.run();
  Ok(harness)
}

fn get_operation<'a>(component: &'a SharedComponent, operation: &str) -> Result<&'a OperationSignature, TestError> {
  component
    .signature()
    .get_operation(operation)
    .ok_or(TestError::OpNotFound(operation.to_owned()))
}

fn validate_config(name: Option<&String>, config: Option<&RuntimeConfig>, fields: &[Field]) -> Result<(), TestError> {
  wick_packet::validation::expect_configuration_matches(name.unwrap_or(&"Test".to_owned()), config, fields)
    .map_err(TestError::ConfigUnsatisfied)
}

#[allow(clippy::too_many_lines)]
async fn run_unit<'a>(
  _i: usize,
  def: &'a mut UnitTest<'a>,
  entity: Entity,
  component: SharedComponent,
  root_config: Option<RuntimeConfig>,
) -> Result<TestBlock, TestError> {
  let span = info_span!("unit test", name = def.test.name());

  let op_config = render_config(def.test.config(), None)?;
  let signature = get_operation(&component, def.test.operation())?;

  validate_config(def.test.name(), op_config.as_ref(), &signature.config)?;

  let (stream, inherent, explicit_done) = get_payload(def, root_config.as_ref(), op_config.as_ref())?;
  let test_name = get_description(def);
  let mut test_block = TestBlock::new(Some(test_name.clone()));
  let prefix = |msg: &str| format!("{}: {}", test_name, if msg.is_empty() { "wick test" } else { msg });

  span.in_scope(|| info!(%entity, "invoke"));

  let invocation = Invocation::new(Entity::test(&test_name), entity, stream, inherent, &span);

  let fut = tokio::time::timeout(
    Duration::from_secs(5),
    component.handle(invocation, op_config.clone(), panic_callback()),
  );

  let result = fut
    .await
    .map_err(|e| TestError::InvocationTimeout(e.to_string()))?
    .map_err(|e| TestError::InvocationFailed(e.to_string()));

  if let Err(e) = result {
    test_block.fail(prefix("invocation"), Some(vec![format!("Invocation failed: {}", e)]));
    return Ok(test_block);
  }

  let stream = result.unwrap();

  let packets = stream
    .collect::<Result<Vec<_>, wick_packet::Error>>()
    .await
    .map_err(|e| TestError::InvocationFailed(e.to_string()))?;

  let mut diagnostics = vec!["Actual Invocation Output (as JSON): ".to_owned()];
  let mut output_lines: Vec<_> = packets.iter().map(|o| format!("{}", o.to_json())).collect();
  diagnostics.append(&mut output_lines);
  test_block.add_diagnostic_messages(diagnostics);

  def.set_actual(packets);

  let mut index = 0;

  let success = loop {
    if index >= def.test.outputs().len() {
      // We've already checked all the outputs, so we're done.
      break true;
    }
    let expected = def.test.outputs().get(index).unwrap();
    let expected = gen_packet(expected, root_config.as_ref(), op_config.as_ref())?;
    if let Err(e) = def.check_next(expected) {
      match e {
        TestError::Assertion(_ex, _act, assertion) => match assertion {
          crate::error::AssertionFailure::Payload(exv, acv) => {
            let diagnostic = assert_json_diff::assert_json_matches_no_panic(
              &acv,
              &exv,
              assert_json_diff::Config::new(assert_json_diff::CompareMode::Inclusive),
            );
            let diagnostic = Some(split_and_indent(&diagnostic.err().unwrap_or_default(), 3));

            test_block.fail(prefix("payload data mismatch"), diagnostic);
          }
          crate::error::AssertionFailure::Flags(exf, acf) => {
            test_block.fail(prefix("flag mismatch"), diag_flags(acf, exf));
          }
          crate::error::AssertionFailure::Name(exn, acf) => {
            test_block.fail(prefix("port name mismatch"), diag_compare(&acf, &exn));
          }
          e @ crate::error::AssertionFailure::ActualNoData => {
            test_block.fail(prefix("port name mismatch"), Some(vec![e.to_string()]));
          }
          e @ crate::error::AssertionFailure::ExpectedNoData => {
            test_block.fail(prefix("port name mismatch"), Some(vec![e.to_string()]));
          }
        },
        e => {
          test_block.fail(prefix("other error"), Some(vec![e.to_string()]));
        }
      }
      break false;
    };

    index += 1;
  };

  if success {
    if let Err(packets) = def.finalize(&explicit_done) {
      test_block.fail(
        prefix("retrieved more packets than test expected."),
        Some(packets.into_iter().map(|p| format!("{:?}", p)).collect()),
      );
    } else {
      test_block.succeed(prefix("invocation succeeded"), None);
    }
  }

  Ok(test_block)
}

fn diag_compare(actual: &str, expected: &str) -> Option<Vec<String>> {
  let mut lines = vec!["Actual: ".to_owned()];
  lines.extend(split_and_indent(actual, 3));
  lines.push("Expected: ".to_owned());
  lines.extend(split_and_indent(expected, 3));
  Some(lines)
}

fn diag_flags(actual: u8, expected: u8) -> Option<Vec<String>> {
  Some(vec![format!("Actual: {}", actual), format!("Expected: {}", expected)])
}

fn split_and_indent(text: &str, spaces: u8) -> Vec<String> {
  let mut lines = vec![];
  for line in text.lines() {
    lines.push(format!("{:spaces$}{}", "", line, spaces = spaces as usize));
  }
  lines
}
