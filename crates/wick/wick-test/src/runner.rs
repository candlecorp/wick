use std::time::Duration;

use flow_component::SharedComponent;
use serde_value::Value;
use tap_harness::{TestBlock, TestRunner};
use tokio_stream::StreamExt;
use wick_packet::{Entity, Invocation, Packet, PacketPayload};

use crate::utils::gen_packet;
use crate::{get_payload, TestError, UnitTest};

#[must_use]
pub fn get_description(test: &UnitTest) -> String {
  format!("{}: test operation '{}'", test.test.name(), test.test.operation())
}

pub async fn run_test<'a, 'b>(
  name: impl AsRef<str> + Sync + Send,
  defs: Vec<&'a mut UnitTest<'a>>,
  id: Option<&'b str>,
  component: SharedComponent,
) -> Result<TestRunner, TestError> {
  let mut harness = TestRunner::new(Some(name));

  for (i, def) in defs.into_iter().enumerate() {
    let entity = id.map_or_else(
      || Entity::local(def.test.operation()),
      |id| Entity::operation(id, def.test.operation()),
    );
    let block = run_unit(i, def, entity, component.clone()).await?;
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
) -> Result<TestBlock, TestError> {
  let (stream, inherent) = get_payload(def);
  let test_name = get_description(def);
  let mut test_block = TestBlock::new(Some(test_name.clone()));
  let prefix = |msg: &str| format!("{}: {}", test_name, msg);

  trace!(i, %entity, "invoke");
  let invocation = Invocation::new(
    Entity::test(&test_name),
    entity,
    stream,
    inherent,
    &tracing::Span::current(),
  );
  let fut = component.handle(
    invocation,
    def.test.config().cloned(),
    std::sync::Arc::new(|_, _, _, _, _, _| panic!()),
  );
  let fut = tokio::time::timeout(Duration::from_secs(5), fut);
  let result = fut
    .await
    .map_err(|e| TestError::InvocationTimeout(e.to_string()))?
    .map_err(|e| TestError::InvocationFailed(e.to_string()));

  if let Err(e) = result {
    test_block.add_test(
      || false,
      prefix("invocation"),
      Some(vec![format!("Invocation failed: {}", e)]),
    );
    return Ok(test_block);
  }

  let stream = result.unwrap();

  let outputs: Vec<_> = stream.filter(|msg| msg.is_ok()).map(|msg| msg.unwrap()).collect().await;
  def.actual = outputs;
  let mut diagnostics = vec!["Output: ".to_owned()];
  let mut output_lines: Vec<_> = def.actual.iter().map(|o| format!("{:?}", o.to_json())).collect();
  diagnostics.append(&mut output_lines);
  test_block.add_diagnostic_messages(diagnostics);

  for (j, expected) in def.test.outputs().iter().cloned().enumerate() {
    if j >= def.actual.len() {
      let diag = Some(vec![
        format!("Trying to test output {:?}", expected),
        format!(
          "But component did not produce any more output. Component produced {} total packets.",
          def.actual.len()
        ),
      ]);
      test_block.add_test(|| false, prefix("stream_length"), diag);
      break;
    }
    let actual = &def.actual[j];
    let result = actual.port() == expected.port();
    let diag = diag_compare(actual.port(), expected.port());
    test_block.add_test(
      move || result,
      prefix(&format!("correct port? ('{}' == '{}')", actual.port(), expected.port())),
      diag,
    );
    let expected = gen_packet(&expected)?;

    let actual_payload = actual.payload.clone();

    if actual.flags() > 0 && actual_payload.bytes().map_or(true, |v| v.is_empty()) {
      let diagnostic = diag_compare(diag_packet(actual), diag_packet(&expected));

      debug!(i,j,actual=?actual, "actual");
      debug!(i,j,expected=?expected, "expected");
      let e_inner = expected.clone();
      let a_inner = actual.clone();
      test_block.add_test(
        move || packet_eq(a_inner, e_inner),
        prefix("actual raw packet == expected raw packet?"),
        diagnostic,
      );
    } else {
      let actual_value: Result<Value, TestError> = actual_payload
        .deserialize()
        .map_err(|e| TestError::ConversionFailed(e.to_string()));
      let expected_value: Result<Value, TestError> = expected
        .deserialize()
        .map_err(|e| TestError::ConversionFailed(e.to_string()));

      let diagnostic = diag_compare(diag_value(&actual_value), diag_value(&expected_value));

      debug!(i,j,actual=?actual_value, "actual");
      debug!(i,j,expected=?expected_value, "expected");
      test_block.add_test(
        move || match (actual_value, expected_value) {
          (Ok(actual), Ok(expected)) => eq(actual, expected),
          (Err(actual), Err(expected)) => err_eq(actual, expected),
          _ => false,
        },
        prefix("actual deserialized payload == expected deserialized payload?"),
        diagnostic,
      );
    }
  }

  let num_tested = def.test.outputs().len();
  let mut missed = vec![];

  for i in num_tested..def.actual.len() {
    if let Some(output) = def.actual.get(i) {
      if output.is_done() {
        debug!(?output, "test missed");
        missed.push(output);
      }
    }
  }
  let num_missed = missed.len();
  test_block.add_test(
    move || num_missed == 0,
    prefix("total_outputs"),
    Some(missed.into_iter().map(|p| format!("{:?}", p)).collect()),
  );

  Ok(test_block)
}

fn diag_value(result: &Result<Value, TestError>) -> String {
  match result {
    Ok(v) => format!("{:?}", v),
    Err(e) => format!("Could not deserialize payload, message was : {}", e),
  }
}

fn diag_packet(packet: &Packet) -> String {
  match &packet.payload {
    PacketPayload::Ok(v) => format!("Ok: {:?} (flags: {:08b})", v, packet.flags()),
    PacketPayload::Err(e) => format!("Err: {} (flags: {:08b})", e.msg(), packet.flags()),
  }
}

fn diag_compare(actual: impl AsRef<str>, expected: impl AsRef<str>) -> Option<Vec<String>> {
  Some(vec![
    format!("Actual: {}", actual.as_ref()),
    format!("Expected: {}", expected.as_ref()),
  ])
}

fn eq(left: Value, right: Value) -> bool {
  promote_val(left) == promote_val(right)
}

#[allow(clippy::needless_pass_by_value)]
fn packet_eq(left: Packet, right: Packet) -> bool {
  left == right
}

#[allow(clippy::needless_pass_by_value)]
fn err_eq(left: TestError, right: TestError) -> bool {
  left == right
}

fn promote_val(val: Value) -> Value {
  match val {
    Value::U8(n) => Value::U64(n.into()),
    Value::U16(n) => Value::U64(n.into()),
    Value::U32(n) => Value::U64(n.into()),
    Value::I8(n) => Value::I64(n.into()),
    Value::I16(n) => Value::I64(n.into()),
    Value::I32(n) => Value::I64(n.into()),
    Value::F32(n) => Value::F64(n.into()),
    Value::Char(n) => Value::String(n.into()),
    x => x,
  }
}

#[cfg(test)]
mod test {
  // tested in the workspace root with a native component.
}
