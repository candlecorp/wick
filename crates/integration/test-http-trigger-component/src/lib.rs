use futures::TryStreamExt;
use wasmrs_guest::StreamExt;
#[cfg(feature = "localgen")]
mod generated;
#[cfg(feature = "localgen")]
use generated as wick;
#[cfg(not(feature = "localgen"))]
mod wick {
  #![allow(unused_imports, missing_debug_implementations, clippy::needless_pass_by_value)]
  wick_component::wick_import!();
}
use wick::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct Request {
  message: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Response {
  output_message: String,
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl HttpHandlerOperation for Component {
  type Error = Box<dyn std::error::Error + Send + Sync>;
  type Outputs = http_handler::Outputs;
  type Config = http_handler::Config;

  async fn http_handler(
    mut request: WickStream<types::http::HttpRequest>,
    body: WickStream<bytes::Bytes>,
    mut outputs: Self::Outputs,
    _ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    if let Some(Ok(request)) = request.next().await {
      println!("{:#?}", request);
    }

    let body: bytes::BytesMut = body.try_collect().await?;
    let res_body = if body.is_empty() {
      bytes::Bytes::new()
    } else {
      let body = String::from_utf8(body.into())?;
      let req: Request = serde_json::from_str(&body)?;
      let new: String = req.message.chars().rev().collect();
      serde_json::to_string(&Response { output_message: new })?.into()
    };
    let mut res = types::http::HttpResponse {
      version: types::http::HttpVersion::Http11,
      status: types::http::StatusCode::Ok,
      headers: std::collections::HashMap::new(),
    };
    res
      .headers
      .insert("Content-Type".to_owned(), vec!["application/json".to_owned()]);
    res
      .headers
      .insert("Access-Control-Allow-Origin".to_owned(), vec!["*".to_owned()]);
    res
      .headers
      .insert("Content-Length".to_owned(), vec![res_body.len().to_string()]);

    println!("SENDING RESPONSE BODY {:#?}", res_body);
    outputs.response.send(&res);
    outputs.body.send(&res_body);
    outputs.response.done();
    outputs.body.done();
    Ok(())
  }
}
