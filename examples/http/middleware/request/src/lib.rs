use std::collections::HashMap;

use wasmrs_guest::*;
mod wick {
  wick_component::wick_import!();
}
use wick::*;
use wick_component::propagate_if_error;

use self::wick::types::http;

#[async_trait::async_trait(?Send)]
impl RedirectOperation for Component {
  type Error = anyhow::Error;
  type Outputs = redirect::Outputs;
  type Config = redirect::Config;

  async fn redirect(
    mut request: WickStream<http::HttpRequest>,
    mut outputs: Self::Outputs,
    _ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let Some(request) = request.next().await {
      let mut request = propagate_if_error!(request, outputs, continue);
      let mut response = http::HttpResponse {
        status: http::StatusCode::Found,
        headers: HashMap::default(),
        version: http::HttpVersion::Http11,
      };
      if request.path == "/redirect" {
        let url = request.query_parameters.get("url").and_then(|v| v.get(0));
        if let Some(url) = url {
          response.headers.insert("Location".to_owned(), vec![url.to_owned()]);
          outputs.response.send(&response);
        }
      } else {
        request
          .headers
          .insert("x-wick-redirect".to_owned(), vec!["false".to_owned()]);
        outputs.request.send(&request);
      }
    }
    outputs.request.done();
    outputs.response.done();
    Ok(())
  }
}

thread_local! {
  static COUNTER: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
}

#[async_trait::async_trait(?Send)]
impl CountOperation for Component {
  type Error = anyhow::Error;
  type Outputs = count::Outputs;
  type Config = count::Config;

  async fn count(
    mut request: WickStream<http::HttpRequest>,
    mut response: WickStream<http::HttpResponse>,
    mut outputs: Self::Outputs,
    _ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let (Some(request), Some(response)) = (request.next().await, response.next().await) {
      let mut response = propagate_if_error!(response, outputs, continue);
      let request = propagate_if_error!(request, outputs, continue);

      let request_mw_header = request
        .headers
        .get("x-wick-redirect")
        .and_then(|v| v.get(0).cloned())
        .unwrap_or_else(|| "n/a".to_owned());
      let count = COUNTER.with(|c| {
        let mut c = c.borrow_mut();
        *c += 1;
        *c
      });
      response
        .headers
        .insert("x-wick-count".to_owned(), vec![count.to_string()]);
      response
        .headers
        .insert("x-wick-redirect".to_owned(), vec![request_mw_header]);
      outputs.response.send(&response);
    }
    outputs.response.done();
    Ok(())
  }
}
