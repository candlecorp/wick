mod test;

use anyhow::Result;
use flow_component::Component;
use pretty_assertions::assert_eq;
use serde_json::json;
use test::*;
use wick_packet::{packets, Packet, RuntimeConfig};

#[test_logger::test(tokio::test)]
async fn test_senders() -> Result<()> {
  first_packet_test("./tests/manifests/v0/core/senders.yaml", Vec::new(), "Hello world").await
}

#[test_logger::test(tokio::test)]
async fn test_pluck() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-pluck.yaml",
    packets!(("input", json!({ "to_pluck" :"Hello world!", "to_ignore": "ignore me" }))),
    "Hello world!",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_pluck_substreams() -> Result<()> {
  test_config(
    "./tests/manifests/v1/core-pluck-streams.yaml",
    None,
    None,
    vec![
      Packet::open_bracket("input"),
      Packet::encode("input", json!({"value": "value1!"})),
      Packet::encode("input", json!({"value": "value2!"})),
      Packet::close_bracket("input"),
      Packet::done("input"),
    ],
    vec![
      Packet::open_bracket("pluck1"),
      Packet::encode("pluck1", "value1!"),
      Packet::encode("pluck1", "value2!"),
      Packet::close_bracket("pluck1"),
      Packet::done("pluck1"),
      Packet::open_bracket("pluck2"),
      Packet::encode("pluck2", "value1!"),
      Packet::encode("pluck2", "value2!"),
      Packet::close_bracket("pluck2"),
      Packet::done("pluck2"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_pluck_shorthand() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-pluck-shorthand.yaml",
    packets!(("request", json!({ "headers" : {"cookie": ["Hello world!"]}}))),
    "Hello world!",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_pluck_shorthand2() -> Result<()> {
  first_packet_test_op(
    "test2",
    "./tests/manifests/v1/core-pluck-shorthand.yaml",
    packets!(("input", json!({ "Raw String Field #" : "Hello world!"}))),
    "Hello world!",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_drop() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-drop.yaml",
    packets!(("first", "first"), ("second", "second"), ("third", "third")),
    "second",
  )
  .await
}

#[test_logger::test(tokio::test)]
// #[ignore]
async fn test_merge() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v1/core-merge.yaml",
    "test",
    packets!(
      ("input_a", "first_value"),
      ("input_b", 2u8),
      ("input_c", ["alpha", "beta"])
    ),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let actual = wrapper.decode_value()?;
  let expected = json!({"a": "first_value", "b": 2, "c": ["alpha", "beta"]});
  assert_eq!(actual, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_sender_merge() -> Result<()> {
  let (interpreter, mut outputs) =
    test::common_setup("./tests/manifests/v1/core-sender-merge.yaml", "test", Vec::new()).await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let actual = wrapper.decode_value()?;
  let expected = json!({"a": true, "b": "Hello world", "c": 123456});
  assert_eq!(actual, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_multi_sender() -> Result<()> {
  let (interpreter, mut outputs) =
    test::common_setup("./tests/manifests/v1/core-multi-sender.yaml", "test", Vec::new()).await?;

  assert_eq!(outputs.len(), 6);

  let _ = outputs.pop();
  let actual = outputs.pop().unwrap().unwrap();

  assert_eq!(actual, Packet::encode("b", "Hello world"));
  let _ = outputs.pop();
  let actual = outputs.pop().unwrap().unwrap();
  assert_eq!(actual, Packet::encode("c", 123456));
  let _ = outputs.pop();
  let actual = outputs.pop().unwrap().unwrap();
  assert_eq!(actual, Packet::encode("a", true));
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_1() -> Result<()> {
  first_packet_test_config(
    "./tests/manifests/v1/core-switch.yaml",
    Some(RuntimeConfig::from_value(json!({"greeting": "Hello"})).unwrap()),
    None,
    packets!(("command", "want_greeting"), ("input", "world")),
    "Hello, world",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_2() -> Result<()> {
  first_packet_test_config(
    "./tests/manifests/v1/core-switch.yaml",
    Some(RuntimeConfig::from_value(json!({"greeting": ""})).unwrap()),
    None,
    packets!(("command", "want_uppercase"), ("input", "hello WORLD")),
    "HELLO WORLD",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_default() -> Result<()> {
  first_packet_test_config(
    "./tests/manifests/v1/core-switch.yaml",
    Some(RuntimeConfig::from_value(json!({"greeting": ""})).unwrap()),
    None,
    packets!(("command", "nomatch"), ("input", "hello WORLD")),
    "hello WORLD",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_bool_true() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-switch-2.yaml",
    packets!(("input", true), ("message", "does not matter")),
    "on_true",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_bool_default() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-switch-2.yaml",
    packets!(("input", false), ("message", "does not matter")),
    "on_false",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_streams() -> Result<()> {
  test_config(
    "./tests/manifests/v1/core-switch-streams.yaml",
    None,
    None,
    vec![
      Packet::encode("input", "first"),
      Packet::open_bracket("message"),
      Packet::encode("message", "first_message_1"),
      Packet::encode("message", "first_message_2"),
      Packet::encode("message", "first_message_3"),
      Packet::close_bracket("message"),
      Packet::encode("input", "second"),
      Packet::open_bracket("message"),
      Packet::encode("message", "second_message_1"),
      Packet::encode("message", "second_message_2"),
      Packet::encode("message", "second_message_3"),
      Packet::close_bracket("message"),
      Packet::encode("input", "neither"),
      Packet::open_bracket("message"),
      Packet::encode("message", "default_message_1"),
      Packet::encode("message", "default_message_2"),
      Packet::encode("message", "default_message_3"),
      Packet::close_bracket("message"),
      Packet::done("input"),
      Packet::done("message"),
    ],
    vec![
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"first_message_1"},
            {"value":"first_message_2"},
            {"value":"first_message_3"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"SECOND_MESSAGE_1"},
            {"value":"SECOND_MESSAGE_2"},
            {"value":"SECOND_MESSAGE_3"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"1_egassem_tluafed"},
            {"value":"2_egassem_tluafed"},
            {"value":"3_egassem_tluafed"},
          ]]
        }),
      ),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_streams_early_matches() -> Result<()> {
  test_config(
    "./tests/manifests/v1/core-switch-streams.yaml",
    None,
    None,
    vec![
      Packet::open_bracket("message"),
      Packet::open_bracket("input"),
      Packet::encode("input", "first"),
      Packet::encode("input", "second"),
      Packet::encode("input", "neither"),
      Packet::close_bracket("input"),
      Packet::open_bracket("message"),
      Packet::encode("message", "first_message_1"),
      Packet::encode("message", "first_message_2"),
      Packet::encode("message", "first_message_3"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::encode("message", "second_message_1"),
      Packet::encode("message", "second_message_2"),
      Packet::encode("message", "second_message_3"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::encode("message", "default_message_1"),
      Packet::encode("message", "default_message_2"),
      Packet::encode("message", "default_message_3"),
      Packet::close_bracket("message"),
      Packet::close_bracket("message"),
      Packet::done("input"),
      Packet::done("message"),
    ],
    vec![
      Packet::open_bracket("output"),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"first_message_1"},
            {"value":"first_message_2"},
            {"value":"first_message_3"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"SECOND_MESSAGE_1"},
            {"value":"SECOND_MESSAGE_2"},
            {"value":"SECOND_MESSAGE_3"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"1_egassem_tluafed"},
            {"value":"2_egassem_tluafed"},
            {"value":"3_egassem_tluafed"},
          ]]
        }),
      ),
      Packet::close_bracket("output"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_streams_multiple_matches() -> Result<()> {
  test_config(
    "./tests/manifests/v1/core-switch-streams.yaml",
    None,
    None,
    vec![
      Packet::open_bracket("input"),
      Packet::open_bracket("message"),
      Packet::encode("input", "first"),
      Packet::encode("input", "second"),
      Packet::encode("input", "neither"),
      Packet::encode("input", "neither2"),
      Packet::close_bracket("input"),
      Packet::open_bracket("message"),
      Packet::encode("message", "first_message_1"),
      Packet::encode("message", "first_message_2"),
      Packet::encode("message", "first_message_3"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::encode("message", "second_message_1"),
      Packet::encode("message", "second_message_2"),
      Packet::encode("message", "second_message_3"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::encode("message", "default_message_1"),
      Packet::encode("message", "default_message_2"),
      Packet::encode("message", "default_message_3"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::encode("message", "default_message_1_2"),
      Packet::encode("message", "default_message_2_2"),
      Packet::encode("message", "default_message_3_2"),
      Packet::close_bracket("message"),
      Packet::close_bracket("message"),
      Packet::done("input"),
      Packet::done("message"),
    ],
    vec![
      Packet::open_bracket("output"),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"first_message_1"},
            {"value":"first_message_2"},
            {"value":"first_message_3"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"SECOND_MESSAGE_1"},
            {"value":"SECOND_MESSAGE_2"},
            {"value":"SECOND_MESSAGE_3"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"1_egassem_tluafed"},
            {"value":"2_egassem_tluafed"},
            {"value":"3_egassem_tluafed"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"2_1_egassem_tluafed"},
            {"value":"2_2_egassem_tluafed"},
            {"value":"2_3_egassem_tluafed"},
          ]]
        }),
      ),
      Packet::close_bracket("output"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_streams_empty_substreams() -> Result<()> {
  test_config(
    "./tests/manifests/v1/core-switch-streams.yaml",
    None,
    None,
    vec![
      Packet::open_bracket("message"),
      Packet::open_bracket("input"),
      Packet::encode("input", "first"),
      Packet::encode("input", "second"),
      Packet::encode("input", "neither"),
      Packet::close_bracket("input"),
      Packet::open_bracket("message"),
      Packet::encode("message", "first_message_1"),
      Packet::encode("message", "first_message_2"),
      Packet::encode("message", "first_message_3"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::encode("message", "default_message_1"),
      Packet::encode("message", "default_message_2"),
      Packet::encode("message", "default_message_3"),
      Packet::close_bracket("message"),
      Packet::close_bracket("message"),
      Packet::done("input"),
      Packet::done("message"),
    ],
    vec![
      Packet::open_bracket("output"),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"first_message_1"},
            {"value":"first_message_2"},
            {"value":"first_message_3"},
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
          ]]
        }),
      ),
      Packet::encode(
        "output",
        json!({
          "message": [[
            {"value":"1_egassem_tluafed"},
            {"value":"2_egassem_tluafed"},
            {"value":"3_egassem_tluafed"},
          ]]
        }),
      ),
      Packet::close_bracket("output"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_streams_default_only() -> Result<()> {
  test_config(
    "./tests/manifests/v1/core-switch-streams-default-only.yaml",
    None,
    None,
    vec![
      Packet::open_bracket("message"),
      Packet::open_bracket("input"),
      Packet::encode("input", "first"),
      Packet::encode("input", "second"),
      Packet::encode("input", "neither"),
      Packet::close_bracket("input"),
      Packet::open_bracket("message"),
      Packet::encode("message", "first_message_1"),
      Packet::encode("message", "first_message_2"),
      Packet::encode("message", "first_message_3"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::close_bracket("message"),
      Packet::open_bracket("message"),
      Packet::encode("message", "default_message_1"),
      Packet::encode("message", "default_message_2"),
      Packet::encode("message", "default_message_3"),
      Packet::close_bracket("message"),
      Packet::close_bracket("message"),
      Packet::done("input"),
      Packet::done("message"),
    ],
    vec![
      Packet::open_bracket("output"),
      Packet::open_bracket("output"),
      Packet::encode("output", "first_message_1"),
      Packet::encode("output", "first_message_2"),
      Packet::encode("output", "first_message_3"),
      Packet::close_bracket("output"),
      Packet::open_bracket("output"),
      Packet::close_bracket("output"),
      Packet::open_bracket("output"),
      Packet::encode("output", "default_message_1"),
      Packet::encode("output", "default_message_2"),
      Packet::encode("output", "default_message_3"),
      Packet::close_bracket("output"),
      Packet::close_bracket("output"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_multi_input_streams() -> Result<()> {
  test_config(
    "./tests/manifests/v1/core-switch-multi-input-streams.yaml",
    None,
    None,
    vec![
      Packet::open_bracket("input"),
      Packet::open_bracket("name"),
      Packet::open_bracket("greeting"),
      Packet::encode("input", "first"),
      Packet::encode("input", "second"),
      Packet::encode("input", "neither"),
      Packet::close_bracket("input"),
      Packet::open_bracket("greeting"),
      Packet::encode("greeting", "Hello, "),
      Packet::encode("greeting", "Bonjour, "),
      Packet::encode("greeting", "Guten tag, "),
      Packet::close_bracket("greeting"),
      Packet::open_bracket("name"),
      Packet::encode("name", "aaa123"),
      Packet::encode("name", "bbb123"),
      Packet::encode("name", "ccc123"),
      Packet::close_bracket("name"),
      Packet::open_bracket("greeting"),
      Packet::encode("greeting", "Hola, "),
      Packet::encode("greeting", "Hi, "),
      Packet::encode("greeting", "Goddag, "),
      Packet::close_bracket("greeting"),
      Packet::open_bracket("name"),
      Packet::encode("name", "ddd123"),
      Packet::encode("name", "eee123"),
      Packet::encode("name", "fff123"),
      Packet::close_bracket("name"),
      Packet::open_bracket("greeting"),
      Packet::encode("greeting", "Salut, "),
      Packet::encode("greeting", "Aloha, "),
      Packet::encode("greeting", "Ciao, "),
      Packet::close_bracket("greeting"),
      Packet::open_bracket("name"),
      Packet::encode("name", "ggg123"),
      Packet::encode("name", "hhh123"),
      Packet::encode("name", "iii123"),
      Packet::close_bracket("name"),
      Packet::close_bracket("name"),
      Packet::close_bracket("greeting"),
      Packet::done("input"),
      Packet::done("name"),
      Packet::done("greeting"),
    ],
    vec![
      Packet::open_bracket("output"),
      Packet::open_bracket("output"),
      Packet::open_bracket("output"),
      Packet::encode("output", "Hello, AAA123"),
      Packet::encode("output", "Bonjour, BBB123"),
      Packet::encode("output", "Guten tag, CCC123"),
      Packet::close_bracket("output"),
      Packet::open_bracket("output"),
      Packet::encode("output", "Hola, 321ddd"),
      Packet::encode("output", "Hi, 321eee"),
      Packet::encode("output", "Goddag, 321fff"),
      Packet::close_bracket("output"),
      Packet::open_bracket("output"),
      Packet::encode("output", "Salut, 321GGG"),
      Packet::encode("output", "Aloha, 321HHH"),
      Packet::encode("output", "Ciao, 321III"),
      Packet::close_bracket("output"),
      Packet::close_bracket("output"),
      Packet::close_bracket("output"),
    ],
  )
  .await
}
