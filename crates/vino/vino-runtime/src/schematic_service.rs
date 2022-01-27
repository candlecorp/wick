pub(crate) mod default;
pub(crate) mod error;
pub(crate) mod input_message;
pub(crate) mod output_message;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::Future;
use parking_lot::RwLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_manifest::parse::CORE_ID;

use error::SchematicError;

use crate::dev::prelude::validator::SchematicValidator;
use crate::dev::prelude::*;
use crate::dispatch::inv_error;
use crate::schematic_service::input_message::InputMessage;
use crate::schematic_service::output_message::OutputMessage;
use crate::transaction::executor::TransactionExecutor;
use crate::transaction::ComponentPayload;
use crate::{CORE_PORT_SEED, VINO_V0_NAMESPACE};

type Result<T> = std::result::Result<T, SchematicError>;

#[derive(Debug)]
pub(crate) struct SchematicService {
  name: String,
  state: RwLock<Option<State>>,
}

#[derive(Debug)]
struct State {
  model: Arc<RwLock<SchematicModel>>,
  transactions: TransactionExecutor,
  executor: HashMap<String, UnboundedSender<TransactionUpdate>>,
  providers: HashMap<String, Arc<ProviderChannel>>,
}

impl Default for SchematicService {
  fn default() -> Self {
    SchematicService {
      name: "".to_owned(),
      state: RwLock::new(None),
    }
  }
}

impl SchematicService {
  pub(crate) fn new(def: &SchematicDefinition) -> Self {
    Self {
      name: def.get_name(),
      ..Default::default()
    }
  }

  pub(crate) fn init(
    &self,
    model: &Arc<RwLock<SchematicModel>>,
    provider_channels: &HashMap<String, Arc<ProviderChannel>>,
    provider_models: HashMap<String, ProviderModel>,
    timeout: Duration,
  ) -> Result<()> {
    let mut model_lock = model.write();
    trace!("SC[{}]:INIT", self.name);

    let allowed_providers = vec![
      model_lock.get_allowed_providers().clone(),
      vec![VINO_V0_NAMESPACE.to_owned(), SELF_NAMESPACE.to_owned()],
    ]
    .concat();
    trace!(
      "SC[{}]:AVAIL_PROVIDERS[{}]:ALLOWED_PROVIDERS[{}]",
      self.name,
      provider_channels.iter().map(|(k, _)| k).join(","),
      allowed_providers.join(",")
    );
    let mut exposed_providers = HashMap::new();
    for provider in allowed_providers {
      match provider_channels.get(&provider) {
        Some(channel) => {
          exposed_providers.insert(provider, channel.clone());
        }
        None => return Err(SchematicError::ProviderNotFound(provider)),
      }
    }

    SchematicValidator::validate_early_errors(&model_lock)?;

    let models: HashMap<_, _> = provider_models
      .into_iter()
      .filter_map(|(ns, model)| exposed_providers.contains_key(&ns).then(|| (ns, model)))
      .collect();

    model_lock.commit_providers(models)?;

    let state = State {
      transactions: TransactionExecutor::new(model.clone(), timeout),
      model: model.clone(),
      executor: HashMap::new(),
      providers: exposed_providers,
    };
    self.state.write().replace(state);
    SchematicValidator::validate_early_errors(&model_lock)?;

    Ok(())
  }

  pub(crate) fn start_tx(
    &self,
    tx_id: String,
  ) -> (UnboundedReceiver<TransactionUpdate>, UnboundedSender<TransactionUpdate>) {
    let mut lock = self.state.write();
    match lock.as_mut() {
      Some(state) => {
        let (rx, tx) = state.transactions.new_transaction(tx_id.clone());
        state.executor.insert(tx_id, tx.clone());
        (rx, tx)
      }
      None => panic!("Internal Error: schematic uninitialized"),
    }
  }

  pub(crate) fn update_tx(&self, tx_id: &str, msg: TransactionUpdate) -> Result<()> {
    let state = self.state.read();
    match state.as_ref() {
      Some(state) => state
        .executor
        .get(tx_id)
        .map(|e| e.send(msg).map_err(|_| InternalError::E6003.into()))
        .ok_or_else(|| SchematicError::TransactionNotFound(tx_id.to_owned()))?,
      None => panic!("Internal Error: schematic uninitialized"),
    }
  }

  pub(crate) fn get_model(&self) -> SharedModel {
    let state = self.state.read();
    match state.as_ref() {
      Some(state) => state.model.clone(),
      None => panic!("Internal Error: schematic uninitialized"),
    }
  }

  pub(crate) fn get_provider_channel(&self, name: &str) -> Result<Arc<ProviderChannel>> {
    let state = self.state.read();
    match state.as_ref() {
      Some(state) => state
        .providers
        .get(name)
        .cloned()
        .ok_or_else(|| SchematicError::InstanceNotFound(name.to_owned())),
      None => panic!("Internal Error: schematic uninitialized"),
    }
  }

  pub(crate) fn get_provider(&self, instance: &str) -> Result<Arc<BoxedInvocationHandler>> {
    let component = get_component_definition(&self.get_model(), instance)?;
    let model = self.get_model();
    let model = model.read();
    let err = SchematicError::InstanceNotFound(instance.to_owned());
    if !model.has_component(&component) {
      warn!("SC[{}]: {} does not have a valid model.", self.name, instance);
      return Err(err);
    }
    trace!("SC[{}]:INSTANCE[{}]->[{}]", self.name, instance, component.id());
    let channel = self.get_provider_channel(&component.namespace)?;
    Ok(channel.recipient.clone())
  }

  pub(crate) async fn output_message(&self, msg: OutputMessage) -> Result<()> {
    let log_prefix = format!("SC[{}]:OUTPUT:{}:", self.name, msg.port);

    let defs = if msg.port.matches_port(vino_transport::COMPONENT_ERROR) {
      error!("{}Component-wide error received: {}", log_prefix, msg.payload);
      get_downstream_connections(&self.get_model(), msg.port.get_instance())
    } else {
      trace!("{}Output ready", log_prefix);
      get_port_connections(&self.get_model(), &msg.port)
    };

    for connection in defs {
      let upstream_port = connection.from.to_string();
      let tx_id = msg.tx_id.clone();
      let next = InputMessage {
        tx_id: tx_id.clone(),
        connection,
        payload: msg.payload.clone(),
      };
      let msg = TransactionUpdate::Update(next.handle_default());

      let send_result = self.update_tx(&tx_id, msg);

      if let Err(e) = send_result {
        debug!("{}ERROR:6001 {:?}", log_prefix, e);
        warn!(
          "Error sending message in transaction {}. This is likely a bug in the upstream (i.e. {})",
          tx_id, upstream_port
        );
      }
    }

    Ok(())
  }

  pub(crate) async fn short_circuit(&self, tx_id: String, instance: String, payload: MessageTransport) -> Result<()> {
    trace!("SC[{}]:{}:BYPASS", self.name, instance);

    let outputs = get_outputs(&self.get_model(), &instance);

    let downstreams: Vec<ConnectionDefinition> = outputs
      .iter()
      .flat_map(|port| get_port_connections(&self.get_model(), port))
      .collect();

    trace!(
      "SC[{}]:{}:BYPASS:Connections {}",
      self.name,
      instance,
      join(&downstreams, ", ")
    );

    for connection in downstreams {
      self
        .output_message(OutputMessage::new(&tx_id, connection.from.clone(), payload.clone()))
        .await?;
      self
        .output_message(OutputMessage::new(&tx_id, connection.from, MessageTransport::done()))
        .await?;
    }

    Ok(())
  }

  pub(crate) async fn component_payload(&self, msg: ComponentPayload) -> Result<()> {
    trace!("SC[{}]:INSTANCE[{}]:READY", self.name, msg.instance);
    let instance = msg.instance.clone();
    let tx_id = msg.tx_id;

    let def = get_component_definition(&self.get_model(), &instance)?;

    if msg.payload_map.has_error() {
      trace!("SC[{}]:INSTANCE[{}]:BYPASSING", self.name, msg.instance);
      let err_payload = msg.payload_map.take_error().unwrap();
      return self.short_circuit(tx_id, instance, err_payload).await;
    }

    let invocation = InvocationMessage::from(Invocation::next(
      &tx_id,
      Entity::schematic(&self.name),
      Entity::component(def.namespace, def.name),
      msg.payload_map,
    ));

    let handler = self.get_provider(&msg.instance)?;

    let sc_name = self.name.clone();

    let target = invocation.get_target_url();

    let response = map_err!(
      tokio::spawn(async move { handler.invoke(invocation)?.await }).await,
      InternalError::E6009
    )??;

    match response {
      InvocationResponse::Stream { tx_id, mut rx } => {
        let log_prefix = format!("SC[{}]:OUTPUT:{}:{}:", sc_name, tx_id, target);
        trace!("{}:STREAM_HANDLER:START", log_prefix,);

        // tokio::spawn(async move {
        while let Some(packet) = rx.next().await {
          let logmsg = format!("ref: {}, port: {}", instance, packet.port);
          let port = ConnectionTargetDefinition::new(instance.clone(), packet.port);
          let msg = OutputMessage {
            port,
            tx_id: tx_id.clone(),
            payload: packet.payload,
          };
          let result = self.output_message(msg).await;
          if result.is_err() {
            error!(
              "{} Error sending output {} {}",
              log_prefix,
              logmsg,
              InternalError::E6013
            );
          }
        }
        trace!("{}:STREAM_HANDLER:COMPLETE", log_prefix);
        // });
        Ok(())
      }
      InvocationResponse::Error { tx_id, msg } => {
        warn!("Tx '{}' short-circuiting '{}': {}", tx_id, instance, msg);
        self.short_circuit(tx_id, instance, MessageTransport::error(msg)).await
      }
    }
  }

  pub(crate) fn invoke(
    self: &Arc<Self>,
    msg: &InvocationMessage,
  ) -> Result<futures::future::BoxFuture<'static, Result<InvocationResponse>>> {
    let tx_id = msg.get_tx_id().to_owned();
    let target = msg.get_target();

    let result = match target {
      Entity::Schematic(name) => handle_schematic(self, name, msg),
      Entity::Component(_, name) => handle_schematic(self, name, msg),
      Entity::Reference(reference) => {
        get_component_definition(&self.get_model(), reference).and_then(|def| handle_schematic(self, &def.id(), msg))
      }
      _ => Err(SchematicError::FailedPreRequestCondition(
        "Schematic invoked with entity it doesn't handle".into(),
      )),
    };

    Ok(
      async move {
        match result {
          Ok(task) => Ok(task.await.map_or_else(|e| inv_error(&tx_id, &e.to_string()), |r| r)),
          Err(e) => Ok(inv_error(&tx_id, &e.to_string())),
        }
      }
      .boxed(),
    )
  }
}

pub(crate) fn handle_schematic(
  schematic: &Arc<SchematicService>,
  name: &str,
  invocation: &InvocationMessage,
) -> Result<impl Future<Output = Result<InvocationResponse>>> {
  let tx_id = invocation.get_tx_id().to_owned();
  let log_prefix = format!("SC[{}]:{}", name, tx_id);
  trace!("{}:INVOKE", log_prefix);

  let (mut outbound, inbound) = schematic.start_tx(tx_id.clone());
  let (tx, rx) = unbounded_channel::<TransportWrapper>();

  let inner = log_prefix;
  let inner_schematic = schematic.clone();
  tokio::spawn(async move {
    while let Some(msg) = outbound.recv().await {
      match msg {
        TransactionUpdate::Done(tx_id) => {
          let output_msg = TransportWrapper {
            payload: MessageTransport::done(),
            port: "<system>".to_owned(),
          };
          if tx.send(output_msg).is_err() {
            warn!("TX:{} {}", tx_id, SchematicError::SchematicClosedEarly);
          }

          trace!("{}:DONE", inner);
          break;
        }
        TransactionUpdate::Result(a) => {
          if let Err(e) = tx.send(a.payload) {
            error!("Error sending result {}", e);
          }
        }
        TransactionUpdate::Execute(a) => {
          if let Err(e) = inner_schematic.component_payload(a).await {
            error!("Error sending execute command {}", e);
          }
        }
        rest => {
          warn!("Unhandled state: {:?}", rest);
        }
      }
    }
    drop(outbound);
    trace!("{}:STOPPING", inner);
  });

  match make_input_packets(name, &schematic.get_model(), &tx_id, invocation) {
    Ok(messages) => {
      for message in messages {
        inbound.send(TransactionUpdate::Update(message.handle_default()))?;
      }
    }
    Err(e) => {
      inbound.send(TransactionUpdate::Error(e.to_string()))?;
      return Err(e);
    }
  };
  let rx = UnboundedReceiverStream::new(rx);

  Ok(async move { Ok(InvocationResponse::stream(tx_id.clone(), rx)) })
}

fn make_input_packets(
  name: &str,
  model: &SharedModel,
  tx_id: &str,
  invocation: &InvocationMessage,
) -> Result<Vec<InputMessage>> {
  let map = invocation.get_payload();
  let model = model.read();
  let connections = model.get_downstream_connections(SCHEMATIC_INPUT);

  let mut messages: Vec<InputMessage> = vec![];
  for conn in connections {
    let transport = map.get(conn.from.get_port()).ok_or_else(|| {
      SchematicError::FailedPreRequestCondition(format!("Port {} not found in transport payload", conn.from))
    })?;
    debug!("SC[{}]:INPUT[{}]:PAYLOAD:{:?}", name, conn.from.get_port(), transport);
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: transport.clone(),
    });
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: MessageTransport::done(),
    });
  }
  let connections = model.get_downstream_connections(CORE_ID);
  for conn in connections {
    let msg = match conn.from.get_port() {
      CORE_PORT_SEED => MessageTransport::success(&invocation.get_init_data().seed),
      x => MessageTransport::error(format!("{} port {} does not exist.", CORE_ID, x)),
    };
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: msg,
    });
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: MessageTransport::done(),
    });
  }

  Ok(messages)
}
