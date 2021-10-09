use std::convert::TryInto;
use std::sync::Arc;

use rpc::invocation_service_client::InvocationServiceClient;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_invocation_server::InvocationClient;
use vino_rpc::rpc;
use vino_rpc::rpc::ListRequest;

use crate::dev::prelude::*;
type Result<T> = std::result::Result<T, ProviderError>;

static PREFIX: &str = "GRPC";

#[derive(Default, Debug)]
pub(crate) struct GrpcProviderService {
  namespace: String,
  state: Option<State>,
  seed: String,
}

#[derive(Debug)]
pub(crate) struct State {
  client: Arc<Mutex<InvocationClient>>,
}

impl OptionalState for GrpcProviderService {
  type State = State;

  fn get_state_option(&self) -> Option<&Self::State> {
    self.state.as_ref()
  }

  fn get_mut_state_option(&mut self) -> Option<&mut Self::State> {
    self.state.as_mut()
  }
}

impl GrpcProviderService {
  pub(crate) fn new(namespace: String, seed: String) -> Self {
    Self {
      namespace,
      seed,
      state: None,
    }
  }

  pub(crate) async fn init(&mut self, address: String) -> Result<()> {
    trace!("{}:Init:{}", PREFIX, self.namespace);

    let address = address;

    let client = InvocationServiceClient::connect(address)
      .await
      .map_err(|e| ProviderError::GrpcUrlProviderError(e.to_string()))?;
    self.state = Some(State {
      client: Arc::new(Mutex::new(client)),
    });
    Ok(())
  }
}

#[async_trait::async_trait]
impl InvocationHandler for GrpcProviderService {
  async fn get_signature(&self) -> Result<ProviderSignature> {
    trace!("{}:InitComponents:[NS:{}]", PREFIX, self.namespace);

    let state = self.get_state()?;

    let list = state
      .client
      .lock()
      .await
      .list(ListRequest {})
      .await
      .map_err(|e| ProviderError::GrpcUrlProviderError(e.to_string()))?;

    let mut list = list.into_inner();
    let sig = list.schemas.remove(0);

    match &sig.r#type {
      Some(rpc::hosted_type::Type::Provider(sig)) => Ok(sig.clone().try_into()?),
      None => Err(InternalError::E7004.into()),
    }
  }

  async fn invoke(&self, msg: InvocationMessage) -> Result<InvocationResponse> {
    trace!(
      "{}:INVOKE:[{}]=>[{}]",
      PREFIX,
      msg.get_origin_url(),
      msg.get_target_url()
    );

    let state = self.get_state()?;
    let client = state.client.clone();
    let tx_id = msg.get_tx_id().to_owned();
    let url = msg.get_target_url();

    let invocation: rpc::Invocation = match msg.try_into() {
      Ok(i) => i,
      Err(_) => {
        return Ok(InvocationResponse::error(
          tx_id.clone(),
          "GRPC provider sent invalid payload".to_owned(),
        ))
      }
    };

    let mut stream = client
      .lock()
      .await
      .invoke(invocation)
      .await
      .map_err(|e| ProviderError::RpcUpstreamError(e.to_string()))?
      .into_inner();
    let (tx, rx) = unbounded_channel();
    trace!("{}[{}]:START", PREFIX, url);
    tokio::spawn(async move {
      loop {
        trace!("{}[{}]:WAIT", PREFIX, url);
        let next = stream.message().await;
        if let Err(e) = next {
          let msg = format!("Error during GRPC stream: {}", e);
          error!("{}", msg);
          match tx.send(TransportWrapper::component_error(MessageTransport::error(
            msg,
          ))) {
            Ok(_) => {
              trace!("Sent error to upstream, closing connection.");
            }
            Err(e) => {
              error!("Error sending output on channel {}", e.to_string());
            }
          }
          break;
        }
        let output = match next.unwrap() {
          Some(v) => v,
          None => break,
        };

        let payload = output.payload;
        if payload.is_none() {
          let msg = "Received response but no payload";
          error!("{}", msg);
          match tx.send(TransportWrapper::component_error(MessageTransport::error(
            msg.to_owned(),
          ))) {
            Ok(_) => {
              trace!("Sent error to upstream");
            }
            Err(e) => {
              error!(
                "Error sending output on channel {}. Closing connection.",
                e.to_string()
              );
              break;
            }
          }
          continue;
        }
        trace!("{}[{}]:PORT[{}]:RECV", PREFIX, url, output.port);

        match tx.send(TransportWrapper {
          port: output.port.clone(),
          payload: payload.unwrap().into(),
        }) {
          Ok(_) => {
            trace!("{}[{}]:PORT[{}]:SENT", PREFIX, url, output.port);
          }
          Err(e) => {
            error!("Error sending output on channel {}", e.to_string());
            break;
          }
        }
      }
      trace!("{}[{}]:FINISH", PREFIX, url);
    });
    let rx = UnboundedReceiverStream::new(rx);
    Ok::<InvocationResponse, ProviderError>(InvocationResponse::stream(tx_id, rx))
  }
}

#[cfg(test)]
mod test {

  use std::time::Duration;

  use actix::clock::sleep;
  use once_cell::sync::Lazy;
  use test_vino_provider::Provider;
  use vino_invocation_server::{
    bind_new_socket,
    make_rpc_server,
  };
  use vino_rpc::BoxedRpcHandler;

  use super::*;
  use crate::test::prelude::assert_eq;
  type Result<T> = super::Result<T>;
  static PROVIDER: Lazy<BoxedRpcHandler> = Lazy::new(|| Box::new(Provider::default()));

  #[test_logger::test(actix_rt::test)]
  async fn test_initialize() -> Result<()> {
    let socket = bind_new_socket()?;
    let port = socket.local_addr()?.port();
    let init_handle = make_rpc_server(socket, &PROVIDER);
    let user_data = "test string payload";

    let mut service = GrpcProviderService::new("test".to_owned(), "seed".to_owned());
    service.init(format!("https://127.0.0.1:{}", port)).await?;

    let work = async move {
      sleep(Duration::from_secs(1)).await;

      let invocation = InvocationMessage::new(
        Entity::test("grpc"),
        Entity::component_direct("test-component"),
        vec![("input", user_data)].into(),
      );

      let response = service.invoke(invocation).await?;
      Ok!(response)
    };
    tokio::select! {
        res = work => {
            debug!("Work complete");
            match res {
              Ok(response)=>{
                let mut rx = response.ok()?;
                let next: TransportWrapper = rx.next().await.unwrap();
                let payload: String = next.payload.try_into()?;
                assert_eq!(payload, format!("TEST: {}", user_data));
              },
              Err(e)=>{
                panic!("task failed: {}", e);
              }
            }
        }
        _ = init_handle => {
            panic!("server died");
        }
    };

    Ok(())
  }
}
