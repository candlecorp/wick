use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use flow_component::RuntimeCallback;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::Span;
use wasmrs::{GenericError, OperationHandler, RSocket, RawPayload};
use wasmrs_codec::messagepack::serialize;
use wasmrs_host::{CallContext, Host, WasiParams};
use wasmrs_rx::{FluxChannel, Observer};
use wick_interface_types::ComponentSignature;
use wick_packet::{
  from_raw_wasmrs,
  from_wasmrs,
  into_wasmrs,
  ComponentReference,
  ContextTransport,
  Entity,
  GenericConfig,
  Invocation,
  PacketStream,
};
use wick_wascap::{Claims, CollectionClaims};

use crate::error::WasmComponentError;
use crate::wasm_module::WickWasmModule;
use crate::{Error, Result};

#[must_use]
pub struct WasmHostBuilder {
  wasi_params: Option<WasiParams>,
  callback: Option<Arc<RuntimeCallback>>,
  min_threads: usize,
  max_threads: usize,
  span: Span,
}

impl std::fmt::Debug for WasmHostBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHostBuilder")
      .field("wasi_params", &self.wasi_params)
      .finish()
  }
}

impl WasmHostBuilder {
  pub fn new(span: Span) -> Self {
    Self {
      wasi_params: None,
      callback: None,
      span,
      min_threads: 1,
      max_threads: 1,
    }
  }

  pub fn wasi_params(mut self, params: WasiParams) -> Self {
    self.wasi_params = Some(params);
    self
  }

  pub fn link_callback(mut self, callback: Arc<RuntimeCallback>) -> Self {
    self.callback = Some(callback);
    self
  }

  pub fn preopened_dirs(mut self, dirs: Vec<String>) -> Self {
    let mut params = self.wasi_params.take().unwrap_or_default();
    params.preopened_dirs = dirs;
    self.wasi_params.replace(params);
    self
  }

  pub fn build(self, module: &WickWasmModule) -> Result<WasmHost> {
    WasmHost::try_load(
      module,
      self.wasi_params,
      &self.callback,
      self.min_threads,
      self.max_threads,
      self.span,
    )
  }

  pub fn max_threads(mut self, max_threads: usize) -> Self {
    self.max_threads = max_threads;
    self
  }

  pub fn min_threads(mut self, min_threads: usize) -> Self {
    self.min_threads = min_threads;
    self
  }
}

#[derive()]
pub struct WasmHost {
  #[allow(unused)]
  host: Arc<Mutex<Host>>,
  claims: Claims<CollectionClaims>,
  ctx: Arc<CallContext>,
  _rng: seeded_random::Random,
  span: Span,
}

impl std::fmt::Debug for WasmHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHost").field("claims", &self.claims).finish()
  }
}

impl WasmHost {
  pub fn try_load(
    module: &WickWasmModule,
    wasi_options: Option<WasiParams>,
    callback: &Option<Arc<RuntimeCallback>>,
    _min_threads: usize,
    _max_threads: usize,
    span: Span,
  ) -> Result<Self> {
    let jwt = &module.token.jwt;
    let _span = span.enter();

    wick_wascap::validate_token::<CollectionClaims>(jwt).map_err(|e| Error::ClaimsInvalid(e.to_string()))?;

    let time = Instant::now();

    let engine = wasmrs_wasmtime::WasmtimeBuilder::new(&module.bytes).enable_cache(None);
    let engine = if let Some(wasi_options) = wasi_options {
      engine.wasi_params(wasi_options)
    } else {
      engine
    };
    let engine = engine
      .build()
      .map_err(|e| WasmComponentError::EngineFailure(e.to_string()))?;
    trace!(duration_μs = %time.elapsed().as_micros(), "wasmtime instance loaded");

    let host = Host::new(engine).map_err(|e| WasmComponentError::EngineFailure(e.to_string()))?;

    debug!(duration_μs = ?time.elapsed().as_micros(), "wasmtime initialize");
    if let Some(callback) = callback {
      let index = host.register_request_channel("wick", "__callback", make_host_callback(callback));
      let cb_span = debug_span!("wasmrs event");
      cb_span.follows_from(&span);
      host.register_fire_and_forget("wick", "__event", make_event_callback(cb_span));
      trace!(index, "wasmrs callback index");
    }
    let ctx = host.new_context(128 * 1024, 128 * 1024).unwrap();

    drop(_span);
    Ok(Self {
      claims: module.claims().clone(),
      host: Arc::new(Mutex::new(host)),
      ctx: Arc::new(ctx),
      _rng: seeded_random::Random::new(),
      span,
    })
  }

  #[allow(clippy::needless_pass_by_value)]
  pub fn call(&self, invocation: Invocation, config: Option<GenericConfig>) -> Result<PacketStream> {
    let _span = self.span.enter();
    let component_name = invocation.target.operation_id();
    let inherent = invocation.inherent;
    let now = Instant::now();
    let ctx = self.ctx.clone();
    let index = ctx
      .get_export("wick", component_name)
      .map_err(|_| crate::Error::OperationNotFound(component_name.to_owned(), ctx.get_exports()))?;

    invocation.packets.set_context(config.unwrap_or_default(), inherent);

    let s = into_wasmrs(index, invocation.packets);
    let out = ctx.request_channel(Box::pin(s));
    trace!(
      component = component_name,
      duration_μs = ?now.elapsed().as_micros(),
      "received stream"
    );
    Ok(from_raw_wasmrs(out))
  }

  pub async fn setup(&self, provided: SetupPayload) -> Result<()> {
    let ctx = self.ctx.clone();
    let payload = self.span.in_scope(|| {
      debug!("wasm setup");

      let index = ctx
        .get_export("wick", "__setup")
        .map_err(|_| crate::Error::SetupOperation)?;
      let metadata = wasmrs::Metadata::new(index);
      let data = serialize(&provided).unwrap();
      Ok::<_, WasmComponentError>(RawPayload::new(metadata.encode(), data.into()))
    })?;

    // this should never take more than a second.
    let result = timeout(Duration::from_millis(1000), ctx.request_response(payload)).await;

    self.span.in_scope(|| {
      match result {
        Ok(Ok(_)) => {
          debug!("setup finished");
        }
        Ok(Err(e)) => {
          error!("setup failed: {}", e);
          return Err(Error::Setup(e));
        }
        Err(e) => {
          error!("setup failed with timeout: {}", e);
          return Err(Error::SetupTimeout);
        }
      }

      trace!("wasm setup finished");
      Ok(())
    })
  }

  pub fn signature(&self) -> &ComponentSignature {
    let claims = &self.claims;
    &claims.metadata.as_ref().unwrap().interface
  }
}

fn make_event_callback(span: Span) -> OperationHandler<wasmrs::IncomingMono, ()> {
  let func = move |incoming: wasmrs::IncomingMono| {
    let span = span.clone();
    tokio::spawn(async move {
      #[allow(clippy::option_if_let_else)]
      if let Ok(payload) = incoming.await {
        span.in_scope(|| debug!("event callback {:?}", payload));
      } else {
        span.in_scope(|| warn!("event callback errored"));
      }
    });
    Ok(())
  };
  Box::new(func)
}

fn make_host_callback(
  rt_cb: &Arc<RuntimeCallback>,
) -> OperationHandler<wasmrs::IncomingStream, wasmrs::OutgoingStream> {
  let cb = rt_cb.clone();
  let span = tracing::info_span!("wasmrs callback");
  let func = move |mut incoming: wasmrs::IncomingStream| -> std::result::Result<wasmrs::OutgoingStream, GenericError> {
    use tokio_stream::StreamExt;
    let (tx, rx) = FluxChannel::new_parts();
    let cb = cb.clone();
    let span = span.clone();
    tokio::spawn(async move {
      let first = incoming.next().await;
      let ctx = if let Some(Ok(first)) = first {
        match wasmrs_codec::messagepack::deserialize::<ContextTransport<Option<GenericConfig>>>(&first.data) {
          Ok(p) => p,
          Err(e) => {
            span.in_scope(|| error!("bad component ref invocation: {}", e));
            let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
            return;
          }
        }
      } else {
        span.in_scope(|| error!("bad component ref invocation: no payload"));
        let _ = tx.error(wick_packet::Error::component_error("no payload"));
        return;
      };
      if ctx.invocation.is_none() {
        span.in_scope(|| error!("bad component ref invocation: no invocation metadata"));
        let _ = tx.error(wick_packet::Error::component_error("no payload"));
        return;
      }
      let config = ctx.config;
      let meta = ctx.invocation.unwrap();
      let stream = from_wasmrs(incoming);
      let inherent = ctx.inherent.next();

      match cb(meta.reference, meta.operation, stream, inherent, config, &span).await {
        Ok(mut response) => {
          while let Some(p) = response.next().await {
            let _ = tx.send_result(p);
          }
        }
        Err(e) => {
          span.in_scope(|| error!("bad component ref invocation: {}", e));
          let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        }
      }
    });
    Ok(into_wasmrs(0, PacketStream::new(Box::new(rx))))
  };
  Box::new(func)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct SetupPayload {
  provided: HashMap<String, ComponentReference>,
  config: GenericConfig,
}

impl SetupPayload {
  pub fn new(origin: &Entity, provided: HashMap<String, String>, config: Option<GenericConfig>) -> Self {
    let provided = provided
      .into_iter()
      .map(|(k, v)| {
        (
          k,
          ComponentReference::new(origin.clone(), Entity::from_str(&v).unwrap()),
        )
      })
      .collect();
    Self {
      provided,
      config: config.unwrap_or_default(),
    }
  }
}
