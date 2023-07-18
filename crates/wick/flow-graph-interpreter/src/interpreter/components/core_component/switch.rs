use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;

use flow_component::{ComponentError, Context, Operation, RenderConfiguration, RuntimeCallback};
use futures::{FutureExt, StreamExt};
use parking_lot::Mutex;
use seeded_random::Seed;
use serde_json::Value;
use tokio::task::{yield_now, JoinHandle};
use tracing::{Instrument, Span};
use wasmrs_rx::Observer;
use wick_interface_types::{Field, OperationSignature, Type};
use wick_packet::{
  ComponentReference,
  Entity,
  InherentData,
  Invocation,
  Packet,
  PacketSender,
  PacketStream,
  RuntimeConfig,
};

use crate::graph::types::{Network, Schematic};
use crate::interpreter::components::self_component::SelfComponent;
use crate::utils::path_to_entity;
use crate::{BoxFuture, HandlerMap};
pub(crate) struct Op {
  signature: Arc<Mutex<Option<OperationSignature>>>,
}

const DISCRIMINANT: &str = "match";

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).finish()
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct Config {
  #[serde(alias = "context")]
  inputs: Vec<Field>,
  outputs: Vec<Field>,
  cases: Vec<SwitchCase>,
  default: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct SwitchCase {
  case: Value,
  #[serde(rename = "do")]
  case_do: String,
  with: Option<RuntimeConfig>,
}

#[allow(clippy::option_if_let_else)]
fn get_op_signature(
  op_path: &str,
  parent_schematic: &Schematic,
  graph: &Network,
  handlers: &HandlerMap,
) -> Option<OperationSignature> {
  if op_path.starts_with("self::") {
    get_self_op_signature(graph, op_path)
  } else if op_path.contains("::") {
    get_handler_op_signature(handlers, op_path)
  } else {
    // if it's a bare name then it must be a node on the parent schematic.
    if let Some(_op) = parent_schematic.nodes().iter().find(|n| n.id() == op_path) {
      panic!("operation instance by the name {} exists, but switch configurations can not delegate to operation instances within a flow. Reference the operation by path instead.", op_path);
      // This is what it points to. We can't simply delegate to this at this point, hence the panic.
      // match op.kind() {
      //   flow_graph::NodeKind::External(ext) => get_ns_op_signature(ext.component_id(), ext.name(), graph, handlers),
      //   _ => None,
      // }
    } else {
      None
    }
  }
}

#[allow(unused)]
fn get_ns_op_signature(ns: &str, op: &str, graph: &Network, handlers: &HandlerMap) -> Option<OperationSignature> {
  if ns == SelfComponent::ID {
    get_self_op_signature(graph, op)
  } else {
    get_handler_op_signature(handlers, op)
  }
}

fn get_handler_op_signature(handlers: &HandlerMap, op_name: &str) -> Option<OperationSignature> {
  op_name.split_once("::").and_then(|(ns, op)| {
    handlers
      .get(ns)
      .and_then(|ns| ns.component.signature().get_operation(op).cloned())
  })
}

fn get_self_op_signature(graph: &Network, schematic_name: &str) -> Option<OperationSignature> {
  let schematic_name = schematic_name.trim_start_matches("self::");
  graph.schematic(schematic_name).map(|schematic| {
    let mut sig = OperationSignature::new(schematic_name);
    for output in schematic.output().outputs() {
      sig = sig.add_output(output.name(), Type::Object); // we don't know the type of the output, so we just use object.
    }
    for input in schematic.input().inputs() {
      sig = sig.add_input(input.name(), Type::Object);
    }
    sig
  })
}

fn log(string: String) -> String {
  error!("{}", string);
  string
}

fn gen_signature(
  id: String,
  parent_schematic: &Schematic,
  graph: &Network,
  handlers: &HandlerMap,
  config: Config,
) -> OperationSignature {
  let mut signature = OperationSignature::new(id);
  signature = signature.add_input(DISCRIMINANT, Type::Object);
  let Some(default_op_sig) = get_op_signature(&config.default, parent_schematic, graph, handlers) else {
    panic!(
      "{}",
      log(format!(
        "Invalid switch configuration: default operation '{}' not found.",
        config.default
      ))
    );
  };
  let case_ops = config
    .cases
    .iter()
    .map(|case| {
      let Some(op_sig) = get_op_signature(&case.case_do, parent_schematic, graph, handlers) else {
        panic!(
          "{}",
          log(format!(
            "Invalid switch configuration: case operation '{}' not found",
            case.case_do
          ))
        );
      };
      op_sig
    })
    .collect::<Vec<_>>();

  let mut default_op_names = default_op_sig.outputs().iter().map(|p| p.name()).collect::<Vec<_>>();
  default_op_names.sort();

  if !case_ops.iter().all(|op| {
    let mut output_names = op.outputs().iter().map(|p| p.name()).collect::<Vec<_>>();
    output_names.sort();
    output_names == default_op_names
  }) {
    error!("The default operation and all case conditions must have the same output signature.");
    panic!();
  }

  for field in config.inputs {
    signature = signature.add_input(field.name, field.ty);
  }
  for field in default_op_sig.outputs {
    signature.outputs.push(field);
  }

  signature
}

impl Op {
  pub(crate) fn new() -> Self {
    Self {
      signature: Default::default(),
    }
  }

  pub(crate) fn gen_signature(
    &self,
    id: String,
    parent_schematic: &Schematic,
    graph: &Network,
    handlers: &HandlerMap,
    config: Config,
  ) -> OperationSignature {
    let sig = gen_signature(id, parent_schematic, graph, handlers, config);
    *self.signature.lock() = Some(sig.clone());
    sig
  }
}

#[derive(Default)]
struct InputStream {
  level: AtomicI32,
  curr_index: AtomicUsize,
  done: AtomicBool,
}

impl InputStream {
  fn level(&self) -> i32 {
    self.level.load(Ordering::Relaxed)
  }

  fn inc_curr_index(&self) {
    let last = self.curr_index.fetch_add(1, Ordering::Relaxed);
    trace!(last, "switch:case: done with condition");
  }

  fn curr_index(&self) -> usize {
    self.curr_index.load(Ordering::Relaxed)
  }

  fn inc_level(&self) {
    let last = self.level.fetch_add(1, Ordering::Relaxed);
    trace!(level = last + 1, "switch:case: incrementing input level");
  }

  fn dec_level(&self) {
    let last = self.level.fetch_sub(1, Ordering::Relaxed);
    trace!(level = last - 1, "switch:case: decrementing input level");
  }

  fn is_done(&self) -> bool {
    self.done.load(Ordering::Relaxed)
  }

  fn set_done(&self) {
    self.done.store(true, Ordering::Relaxed);
  }
}

#[derive()]
struct SwitchRouter {
  conditions: VecDeque<Condition>,
  buffer: VecDeque<Packet>,
  raw_buffer: VecDeque<Packet>,
  num_conditions: Option<usize>,
  span: Span,
}

impl SwitchRouter {
  fn new(span: Span) -> Self {
    Self {
      conditions: Default::default(),
      buffer: Default::default(),
      raw_buffer: Default::default(),
      num_conditions: None,
      span,
    }
  }
  fn pop_buffer(&mut self) -> Option<Packet> {
    self.buffer.pop_front()
  }

  fn buffer_raw(&mut self, packet: Packet) {
    self.raw_buffer.push_back(packet);
  }

  fn push(&mut self, condition: Condition) {
    self.conditions.push_back(condition);
  }

  fn condition_level(&self, index: usize) -> i32 {
    self.conditions.get(index).map_or(0, |c| c.level)
  }

  fn get(&self, index: usize) -> Option<&Condition> {
    self.conditions.get(index)
  }

  fn freeze(&mut self) {
    self.span.in_scope(|| {
      if self.conditions.is_empty() {
        debug!("switch:case: match stream done with no conditions set");
      } else {
        trace!(conditions = self.conditions.len(), "switch:case: freezing conditions");
      }
    });
    self.num_conditions = Some(self.conditions.len());
  }

  fn is_ready(&self, index: usize) -> bool {
    self.get(index).is_some()
  }

  fn can_handle(&self, index: usize) -> CanHandle {
    if self.is_ready(index) {
      CanHandle::Yes
    } else if !self.is_frozen() {
      CanHandle::Maybe
    } else {
      CanHandle::No
    }
  }

  fn is_frozen(&self) -> bool {
    self.num_conditions.is_some()
  }

  fn handle_packet(&mut self, index: usize, packet: Packet) {
    let condition = self.get(index).unwrap(); // unwrap ok because we check is_ready first.
    self
      .span
      .in_scope(|| debug!(condition=%condition.value, ?packet, "switch:case: routing packet to case"));
    condition.handler.send(packet);
  }

  async fn cleanup(self, fields: &[Field], tx: &PacketSender) {
    for mut condition in self.conditions {
      self
        .span
        .in_scope(|| debug!(case=?condition.value, "switch:case: cleaning up case"));
      condition.handler.finish(fields, tx).instrument(self.span.clone()).await;
    }
  }

  async fn finish(&mut self, index: usize, fields: &[Field], tx: &PacketSender) {
    if let Some(condition) = self.conditions.get_mut(index) {
      self
        .span
        .in_scope(|| debug!(case=?condition.value, "switch:case: finishing case iteration"));
      condition.handler.finish(fields, tx).instrument(self.span.clone()).await;
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanHandle {
  Yes,
  No,
  Maybe,
}

struct RouteHandler {
  tx: Option<PacketSender>,
  rx: Option<PacketStream>,
  task: Option<JoinHandle<()>>,
}

impl RouteHandler {
  fn new(sender: PacketSender, receiver: PacketStream, task: JoinHandle<()>) -> Self {
    Self {
      tx: Some(sender),
      rx: Some(receiver),
      task: Some(task),
    }
  }

  fn send(&self, packet: Packet) {
    let _ = self.tx.as_ref().unwrap().send(packet);
  }

  async fn finish(&mut self, fields: &[Field], tx: &PacketSender) {
    if let Some(tx) = self.tx.take() {
      for field in fields {
        debug!(input= %field.name,"switch:case: sending done to case stream");
        let _ = tx.send(Packet::done(field.name()));
      }
    }
    if let Some(task) = self.task.take() {
      debug!("switch:case: awaiting case task completion");
      let _ = task.await;
    }

    if let Some(mut rx) = self.rx.take() {
      debug!("switch:case: draining case stream");
      while let Some(packet) = rx.next().await {
        let _ = tx.send_result(packet);
      }
    }
  }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
enum CaseId<'a> {
  Match(CaseValue<'a>),
  Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CaseValue<'a>(&'a Value);

impl<'a> Hash for CaseValue<'a> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    hash_value(self.0, state);
  }
}

fn hash_value<H: std::hash::Hasher>(val: &Value, state: &mut H) {
  match val {
    Value::Null => None::<()>.hash(state),
    Value::Bool(v) => v.hash(state),
    Value::Number(n) => n.hash(state),
    Value::String(v) => v.hash(state),
    Value::Array(v) => {
      for el in v {
        hash_value(el, state);
      }
    }
    Value::Object(v) => {
      for (k, v) in v {
        k.hash(state);
        hash_value(v, state);
      }
    }
  }
}

struct Condition {
  level: i32,
  value: Value,
  handler: RouteHandler,
}

impl Condition {
  fn new(value: Value, level: i32, handler: RouteHandler) -> Self {
    trace!(%value,level,"switch:case: creating condition");
    Self { level, value, handler }
  }
}

impl Operation for Op {
  const ID: &'static str = "switch";
  type Config = Config;

  #[allow(clippy::too_many_lines)]
  fn handle(
    &self,
    mut invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let (root_tx, root_rx) = invocation.make_response();

    let default = context.config.default.clone();
    let callback = context.callback;

    tokio::spawn(async move {
      // the substream level the condition was found at.
      let mut condition_level = 0;
      let mut router = SwitchRouter::new(invocation.span.clone());
      let mut root_stream = invocation.eject_stream();

      let input_streams: HashMap<String, InputStream> = context
        .config
        .inputs
        .iter()
        .map(|i| (i.name.clone(), InputStream::default()))
        .collect();

      let rng = seeded_random::Random::from_seed(Seed::unsafe_new(invocation.seed()));

      'outer: loop {
        // if we don't yet have any condition, hold on to our input packets.
        let packet = match router.pop_buffer() {
          Some(p) => Some(Ok(p)),
          None => root_stream.next().await,
        };

        #[cfg(debug_assertions)]
        invocation.trace(|| trace!(?packet, "switch:stream:packet"));

        let packet = match packet {
          Some(Ok(p)) => p,
          Some(Err(e)) => {
            let _ = root_tx.error(e);
            continue;
          }
          None => {
            if router.raw_buffer.is_empty() {
              break 'outer;
            }
            router.buffer.extend(router.raw_buffer.drain(0..));
            continue;
          }
        };

        // if this is a packet on the DISCRIMINANT port, decode it and set the condition.
        if packet.port() == DISCRIMINANT {
          if packet.has_data() {
            let condition = packet.decode_value().unwrap();
            let case = context.config.cases.iter().find(|case| case.case == condition);
            let (_case, op, op_config) = case.map_or_else(
              || {
                invocation
                  .trace(|| trace!(case = "default", condition = %condition, op = default, "switch:case:condition"));
                (CaseId::Default, &default, None)
              },
              |case| {
                invocation.trace(
                  || trace!(case = %case.case, condition = %condition, op = case.case_do, "switch:case:condition"),
                );
                (CaseId::Match(CaseValue(&case.case)), &case.case_do, case.with.clone())
              },
            );
            let span = invocation.following_span(trace_span!("switch:case:handler",%condition));
            router.push(Condition::new(
              condition,
              condition_level,
              new_route_handler(
                path_to_entity(op),
                &invocation,
                InherentData::new(rng.gen(), invocation.timestamp()),
                callback.clone(),
                op_config,
                span,
              ),
            ));
            // Now that we have a new condition, re-process all buffered packets.
            router.buffer.extend(router.raw_buffer.drain(0..));
          } else if packet.is_done() {
            router.freeze();
          } else if packet.is_open_bracket() {
            condition_level += 1;
          } else if packet.is_close_bracket() {
            condition_level -= 1;
            assert!(condition_level >= 0, "Received close bracket without open bracket");
          }
          continue;
        }

        let Some(incoming_input) = input_streams.get(packet.port()) else {
          invocation.trace(|| {
            warn!(
              port = packet.port(),
              "switch:stream: received packet on unrecognized input port"
            );
          });
          continue;
        };

        // if this is a done packet on an input port, then mark that input as done.
        if packet.is_done() {
          invocation.trace(|| trace!(port = packet.port(), "switch:stream: input stream done"));
          incoming_input.set_done();
          continue;
        }

        match router.can_handle(incoming_input.curr_index()) {
          CanHandle::No => {
            invocation.trace(|| trace!(input = packet.port(), "switch:stream: routing packet to root stream"));
            for output in context.config.outputs.iter() {
              let _ = root_tx.send(packet.clone().set_port(output.name()));
            }
            continue;
          }
          CanHandle::Maybe => {
            invocation.trace(|| {
              trace!(
                input = packet.port(),
                index = incoming_input.curr_index(),
                "switch:case: buffering packet"
              );
            });
            router.buffer_raw(packet);
            continue;
          }
          _ => {}
        }

        // if this is an open bracket, then we need to decide where to route it.
        if packet.is_open_bracket() {
          incoming_input.inc_level();

          // if we're more nested than the condition level, route it to the case handler.
          if incoming_input.level() > router.condition_level(incoming_input.curr_index()) {
            router.handle_packet(incoming_input.curr_index(), packet);
          } else {
            // otherwise, send the packet to the root stream.
            invocation.trace(|| debug!("switch:stream: routing open bracket to root stream"));
            for output in context.config.outputs.iter() {
              let _ = root_tx.send(Packet::open_bracket(output.name()));
            }
          }
          continue;
        }

        // if this is a close bracket, we need to decide if we're still routing it to the case stream.
        if packet.is_close_bracket() {
          incoming_input.dec_level();
          let level = incoming_input.level();
          let curr_index = incoming_input.curr_index();
          let condition_level = router.condition_level(curr_index);

          match level.cmp(&condition_level) {
            std::cmp::Ordering::Greater => {
              // If we're more nested than the condition level, route it to the case.
              router.handle_packet(curr_index, packet);
            }
            std::cmp::Ordering::Equal => {
              // If we're back at the condition level, then this input has moved on with this condition.
              router.handle_packet(curr_index, packet);
              incoming_input.inc_curr_index();

              if input_streams.iter().all(|(_, v)| v.curr_index() > curr_index) {
                // If all inputs have moved on, finish the handler.
                router.finish(curr_index, &context.config.inputs, &root_tx).await;
                if input_streams.iter().all(|(_, v)| v.is_done()) {
                  invocation.trace(|| trace!("switch: all inputs done"));
                  break 'outer;
                }
              }
            }
            std::cmp::Ordering::Less => {
              // Otherwise, send the packet to the root stream.
              invocation.trace(|| trace!("switch:stream: routing close bracket to root stream"));
              for output in context.config.outputs.iter() {
                let _ = root_tx.send(Packet::close_bracket(output.name()));
              }
            }
          }

          continue;
        }

        // otherwise, send the packet to the case stream.
        router.handle_packet(incoming_input.curr_index(), packet);
        yield_now().await;
      }

      invocation.trace(|| trace!("switch:stream: all inputs done and buffers drained"));

      router.cleanup(&context.config.inputs, &root_tx).await;

      // Send done packets for all defined outputs to our root stream.
      for output in context.config.outputs.iter() {
        invocation.trace(|| trace!(port = output.name(), "switch:stream: sending done to root stream"));
        let _ = root_tx.send(Packet::done(output.name()));
      }
    });

    async move { Ok(root_rx) }.boxed()
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    panic!("{} operation has a dynamic signature", Self::ID);
  }

  fn input_names(&self, config: &Self::Config) -> Vec<String> {
    let mut context: Vec<_> = config.inputs.iter().map(|n| n.name.clone()).collect();
    context.push(DISCRIMINANT.to_owned());
    context
  }
}

fn new_route_handler(
  target: Entity,
  invocation: &Invocation,
  inherent: InherentData,
  callback: Arc<RuntimeCallback>,
  op_config: Option<RuntimeConfig>,
  span: Span,
) -> RouteHandler {
  let op_id = target.operation_id().to_owned();

  let (outer_tx, outer_rx) = invocation.make_response();
  let (inner_tx, inner_rx) = invocation.make_response();
  let compref = ComponentReference::new(invocation.target.clone(), target);

  span.in_scope(|| trace!(%compref,"switch:case: route handler created"));
  let handle = tokio::spawn(async move {
    let call = compref.to_string();
    span.in_scope(|| trace!(invocation = %call, state="starting", "switch:case:task"));
    match callback(compref, op_id, inner_rx, inherent, op_config, &span).await {
      Ok(mut inv_stream) => {
        span.in_scope(|| trace!(invocation = %call, state="starting", "switch:case:stream"));
        while let Some(packet) = inv_stream.next().await {
          span.in_scope(|| trace!(invocation = %call, ?packet, "switch:case:stream"));

          if let Ok(packet) = &packet {
            if packet.is_done() {
              break;
            }
          }
          let _ = outer_tx.send_result(packet);
        }
        span.in_scope(|| trace!(invocation = %call, state="done", "switch:case:stream"));
      }
      Err(e) => {
        span.in_scope(|| warn!(err=%e, "switch:case:error"));
        let _ = outer_tx.error(wick_packet::Error::component_error(e.to_string()));
      }
    };
    span.in_scope(|| trace!(invocation = %call, state="done", "switch:case:task"));
  });
  RouteHandler::new(inner_tx, outer_rx, handle)
}

impl RenderConfiguration for Op {
  type Config = Config;
  type ConfigSource = RuntimeConfig;

  fn decode_config(data: Option<Self::ConfigSource>) -> Result<Self::Config, ComponentError> {
    let config = data.ok_or_else(|| {
      ComponentError::message("Switch component requires configuration, please specify configuration.")
    })?;
    Ok(Self::Config {
      inputs: config
        .coerce_key("context")
        .or_else(|_| config.coerce_key("inputs"))
        .map_err(ComponentError::new)?,
      outputs: config.coerce_key("outputs").map_err(ComponentError::new)?,
      cases: config.coerce_key("cases").map_err(ComponentError::new)?,
      default: config.coerce_key("default").map_err(ComponentError::new)?,
    })
  }
}
