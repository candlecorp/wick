#[allow(unused, clippy::derivable_impls)]
mod import_types;
mod test1 {
  use crate::import_types::*;
  # [cfg_attr (target_family = "wasm" , async_trait :: async_trait (? Send))]
  #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
  impl testop::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = testop::Outputs;
    type Config = testop::Config;

    #[allow(unused)]
    async fn testop(
      message: WickStream<types::http::HttpResponse>,
      outputs: Self::Outputs,
      ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
      Ok(())
    }
  }

  # [cfg_attr (target_family = "wasm" , async_trait :: async_trait (? Send))]
  #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
  impl echo::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = echo::Outputs;
    type Config = echo::Config;

    #[allow(unused)]
    async fn echo(
      message: WickStream<types::http::HttpRequest>,
      time: WickStream<datetime::DateTime>,
      outputs: Self::Outputs,
      ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
      Ok(())
    }
  }

  #[cfg(test)]
  mod test {

    use anyhow::Result;
    use flow_component::{panic_callback, Context};
    use types::http;
    use wick_component::datetime::TimeZone;
    use wick_component::*;
    use wick_packet::{ContextTransport, FluxChannel, InherentData};

    use super::*;
    use crate::import_types::testop::Operation;
    use crate::import_types::types::http::HttpResponse;

    #[tokio::test]
    async fn test_typegen() -> Result<()> {
      let date = datetime::Utc.with_ymd_and_hms(2023, 6, 8, 17, 2, 52).unwrap();
      let date_str = date.to_rfc3339();

      println!("date: {}", date_str);

      let expected = types::LocalStruct {
        field1: "value".to_owned(),
        inner: types::LocalStructInner {
          field1: "value".to_owned(),
          field2: "value".to_owned(),
        },
        time: date,
      };

      let bytes = wasmrs_codec::messagepack::serialize(&expected).unwrap();

      let actual: types::LocalStruct = wasmrs_codec::messagepack::deserialize(&bytes).unwrap();

      assert_eq!(expected, actual);

      let json = serde_json::json!({
        "field1": "value",
        "inner": {
          "field1": "value",
          "field2": "value"
        },
        "time":date.timestamp_millis()
      });

      let actual: types::LocalStruct = serde_json::from_value(json).unwrap();

      assert_eq!(expected, actual);

      let expected = types::LocalUnion::String("Helloooo World".to_owned());

      let bytes = wasmrs_codec::messagepack::serialize(&expected).unwrap();

      let actual: types::LocalUnion = wasmrs_codec::messagepack::deserialize(&bytes).unwrap();

      assert_eq!(expected, actual);

      let json = serde_json::json!("Helloooo World");

      let actual: types::LocalUnion = serde_json::from_value(json).unwrap();

      assert_eq!(expected, actual);

      let response = HttpResponse {
        version: http::HttpVersion::Http11,
        status: http::StatusCode::Ok,
        headers: Default::default(),
      };
      let packets = once(response);
      let tx = FluxChannel::new();
      let outputs = testop::Outputs::new(tx);
      let ctx = Context::new(
        testop::Config::default(),
        &InherentData::unsafe_default(),
        panic_callback(),
      );

      Component::testop(Box::pin(packets), outputs, ctx).await?;
      Ok(())
    }

    #[tokio::test]
    async fn test_configgen() -> Result<()> {
      // Don't delete, it tests that local structs are genned correctly.
      let config = testop::Config {
        a: "value".to_owned(),
        b: 2,
      };
      let expected = ContextTransport::new(config, InherentData::unsafe_default());
      let bytes = wasmrs_codec::messagepack::serialize(&expected).unwrap();
      let actual: ContextTransport<testop::Config> = wasmrs_codec::messagepack::deserialize(&bytes).unwrap();
      assert_eq!(actual.config, expected.config);

      Ok(())
    }
  }
}
