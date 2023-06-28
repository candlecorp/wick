use std::time::Duration;

use flow_component::{panic_callback, SharedComponent};
use serde::Deserialize;
use serde_value::Value;
use tap_harness::{TestBlock, TestRunner};
use tokio_stream::StreamExt;
use wick_packet::{Entity, Invocation, Packet, PacketPayload, RuntimeConfig};

use crate::utils::{gen_packet, render_config};
use crate::{get_payload, TestError, UnitTest};

#[must_use]
pub fn get_description(test: &UnitTest) -> String {
  format!(
    "(test name='{}', operation='{}')",
    test.test.name(),
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

#[allow(clippy::too_many_lines)]
async fn run_unit<'a>(
  i: usize,
  def: &'a mut UnitTest<'a>,
  entity: Entity,
  component: SharedComponent,
  root_config: Option<RuntimeConfig>,
) -> Result<TestBlock, TestError> {
  let span = debug_span!("unit test", name = def.test.name());
  let op_config = render_config(def.test.config(), None)?;
  let signature = component
    .signature()
    .get_operation(def.test.operation())
    .ok_or(TestError::OpNotFound(def.test.operation().to_owned()))?;
  wick_packet::validation::expect_configuration_matches(def.test.name(), op_config.as_ref(), &signature.config)
    .map_err(TestError::ConfigUnsatisfied)?;
  let (stream, inherent) = get_payload(def, root_config.as_ref(), op_config.as_ref())?;
  let test_name = get_description(def);
  let mut test_block = TestBlock::new(Some(test_name.clone()));
  let prefix = |msg: &str| format!("{}: {}", test_name, if msg.is_empty() { "wick test" } else { msg });

  span.in_scope(|| info!(%entity, "invoke"));
  let invocation = Invocation::new(Entity::test(&test_name), entity, stream, inherent, &span);
  let fut = component.handle(invocation, op_config.clone(), panic_callback());
  let fut = tokio::time::timeout(Duration::from_secs(5), fut);
  let result = fut
    .await
    .map_err(|e| TestError::InvocationTimeout(e.to_string()))?
    .map_err(|e| TestError::InvocationFailed(e.to_string()));

  if let Err(e) = result {
    test_block.fail(prefix("invocation"), Some(vec![format!("Invocation failed: {}", e)]));
    return Ok(test_block);
  }

  let stream = result.unwrap();

  let outputs: Vec<_> = stream.filter(|msg| msg.is_ok()).map(|msg| msg.unwrap()).collect().await;
  def.actual = outputs;
  let mut diagnostics = vec!["Actual Invocation Output (as JSON): ".to_owned()];
  let mut output_lines: Vec<_> = def.actual.iter().map(|o| format!("{}", o.to_json())).collect();
  diagnostics.append(&mut output_lines);
  test_block.add_diagnostic_messages(diagnostics);

  let mut index = 0;

  let success = loop {
    if index > def.test.outputs().len() - 1 {
      // We've already checked all the outputs, so we're done.
      break true;
    }
    let expected = def.test.outputs().get(index).unwrap();
    if index >= def.actual.len() {
      let diag = Some(vec![
        format!("Trying to test output {:?}", expected),
        format!(
          "But component did not produce any more output. Component produced {} total packets.",
          def.actual.len()
        ),
      ]);
      test_block.fail(prefix("expected more packets than invocation produced"), diag);
      break false;
    }
    let actual = &def.actual[index];
    let diag = diag_compare(actual.port(), expected.port());
    if actual.port() != expected.port() {
      test_block.fail(
        prefix(&format!(
          "got packet on unexpected port (got {}, expected {})",
          actual.port(),
          expected.port()
        )),
        diag,
      );
      break false;
    }
    let expected = gen_packet(expected, root_config.as_ref(), op_config.as_ref())?;

    let actual_payload = actual.payload.clone();

    if actual.flags() > 0 && actual_payload.bytes().map_or(true, |v| v.is_empty()) {
      let diagnostic = diag_compare(&diag_packet(actual), &diag_packet(&expected));

      debug!(i,index,actual=?actual, "actual");
      debug!(i,index,expected=?expected, "expected");
      if !packet_eq(actual, &expected) {
        test_block.fail(prefix("packet data mismatch"), diagnostic);
        break false;
      }
    }

    let actual_value: Result<Value, TestError> = actual_payload
      .decode()
      .map_err(|e| TestError::Deserialization(e.to_string()));
    let expected_value: Result<Value, TestError> =
      expected.decode().map_err(|e| TestError::Deserialization(e.to_string()));

    debug!(i,index,actual=?actual_value, "actual");
    debug!(i,index,expected=?expected_value, "expected");

    let desc = prefix("payload data mismatch");

    let (success, diagnostic) = match (&actual_value, &expected_value) {
      (Ok(actual), Ok(expected)) => {
        let diagnostic = assert_json_diff::assert_json_matches_no_panic(
          &actual,
          &expected,
          assert_json_diff::Config::new(assert_json_diff::CompareMode::Inclusive),
        );

        (
          eq(actual, expected),
          Some(split_and_indent(&diagnostic.err().unwrap_or_default(), 3)),
        )
      }
      (Err(actual), Err(expected)) => (
        err_eq(actual, expected),
        diag_compare(&diag_result(actual_value), &diag_result(expected_value)),
      ),
      _ => (
        false,
        diag_compare(&diag_result(actual_value), &diag_result(expected_value)),
      ),
    };

    if !success {
      test_block.fail(desc, diagnostic);
      break false;
    }

    index += 1;
  };

  let num_tested = def.test.outputs().len();
  let mut missed = vec![];

  if success {
    // make sure we've tested all the outputs.
    for i in num_tested..def.actual.len() {
      if let Some(output) = def.actual.get(i) {
        if output.has_data() {
          debug!(?output, "test missed");
          missed.push(output);
        }
      }
    }
    let num_missed = missed.len();
    if num_missed > 0 {
      test_block.fail(
        prefix("retrieved more significant output packets than test expected."),
        Some(missed.into_iter().map(|p| format!("{:?}", p)).collect()),
      );
    } else {
      test_block.succeed(prefix("invocation succeeded"), None);
    }
  }

  Ok(test_block)
}

fn diag_result(result: Result<Value, TestError>) -> String {
  match result {
    Ok(v) => serde_json::to_string_pretty(&serde_json::Value::deserialize(v).unwrap()).unwrap(),
    Err(e) => format!("Could not deserialize payload, message was : {}", e),
  }
}

fn diag_packet(packet: &Packet) -> String {
  match &packet.payload {
    PacketPayload::Ok(_) => format!("Success packet w/flags: {:08b}", packet.flags()),
    PacketPayload::Err(e) => format!("Error packet w/flags: {:08b} & message: {}", packet.flags(), e.msg()),
  }
}

fn diag_compare(actual: &str, expected: &str) -> Option<Vec<String>> {
  let mut lines = vec!["Actual: ".to_owned()];
  lines.extend(split_and_indent(actual, 3));
  lines.push("Expected: ".to_owned());
  lines.extend(split_and_indent(expected, 3));
  Some(lines)
}

fn split_and_indent(text: &str, spaces: u8) -> Vec<String> {
  let mut lines = vec![];
  for line in text.lines() {
    lines.push(format!("{:spaces$}{}", "", line, spaces = spaces as usize));
  }
  lines
}

fn eq(left: &Value, right: &Value) -> bool {
  promote_val(left) == promote_val(right)
}

fn packet_eq(left: &Packet, right: &Packet) -> bool {
  left == right
}

#[allow(clippy::needless_pass_by_value)]
fn err_eq(left: &TestError, right: &TestError) -> bool {
  left == right
}

fn promote_val(val: &Value) -> Value {
  match val {
    Value::U8(n) => Value::U64((*n).into()),
    Value::U16(n) => Value::U64((*n).into()),
    Value::U32(n) => Value::U64((*n).into()),
    Value::I8(n) => Value::I64((*n).into()),
    Value::I16(n) => Value::I64((*n).into()),
    Value::I32(n) => Value::I64((*n).into()),
    Value::F32(n) => Value::F64((*n).into()),
    Value::Char(n) => Value::String((*n).into()),
    x => x.clone(),
  }
}

#[cfg(test)]
mod test {
  // tested in the workspace root with a native component.
}
