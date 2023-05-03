use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use flow_component::{ComponentError, Context, Operation};
use futures::{FutureExt, StreamExt};
use parking_lot::Mutex;
use wasmrs_rx::Observer;
use wick_interface_types::{Field, OperationSignature, TypeSignature};
use wick_packet::{ComponentReference, Entity, PacketSender, PacketStream};

use crate::constants::NS_CORE;
use crate::graph::types::Network;
use crate::utils::path_to_entity;
use crate::BoxFuture;
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
  context: Vec<Field>,
  outputs: Vec<Field>,
  cases: Vec<SwitchCase>,
  default: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct SwitchCase {
  case: String,
  #[serde(rename = "do")]
  case_do: String,
}

fn gen_signature(id: &str, graph: &Network, config: Config) -> OperationSignature {
  let mut signature = OperationSignature::new(id);
  signature = signature.add_input(DISCRIMINANT, TypeSignature::Object);
  let default_op_path = config.default.trim_start_matches("self::");
  let default_op = graph.schematic(default_op_path).unwrap_or_else(|| {
    error!(
      "Invalid switch configuration: default operation '{}' not found.",
      default_op_path
    );
    panic!();
  });
  let case_ops = config
    .cases
    .iter()
    .map(|case| {
      graph
        .schematic(case.case_do.trim_start_matches("self::"))
        .unwrap_or_else(|| {
          error!(
            "Invalid switch configuration: case operation '{}' not found",
            case.case_do
          );
          panic!();
        })
        .clone()
    })
    .collect::<Vec<_>>();

  if !case_ops
    .iter()
    .all(|op| op.output().outputs() == default_op.output().outputs())
  {
    error!("The default operation and all case conditions must have the same output signature.");
    panic!();
  }

  for field in config.context {
    signature = signature.add_input(field.name, field.ty);
  }
  for field in config.outputs.clone() {
    signature = signature.add_output(field.name, field.ty);
  }

  signature
}

impl Op {
  pub(crate) fn new() -> Self {
    Self {
      signature: Default::default(),
    }
  }

  pub(crate) fn gen_signature(&self, graph: &Network, config: Config) -> OperationSignature {
    let sig = gen_signature(Op::ID, graph, config);
    *self.signature.lock() = Some(sig.clone());
    sig
  }
}

impl Operation for Op {
  const ID: &'static str = "switch";
  type Config = Config;
  fn handle(
    &self,
    mut stream: PacketStream,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let (tx, rx) = PacketStream::new_channels();

    let default = context.config.default.clone();
    let callback = context.callback;
    tokio::spawn(async move {
      let mut condition = None;
      let mut held_packets = VecDeque::new();
      let mut router: HashMap<String, PacketSender> = HashMap::new();
      let origin = Entity::operation(NS_CORE, Op::ID);
      loop {
        let packet = match held_packets.pop_front() {
          Some(p) => p,
          None => match stream.next().await {
            Some(Ok(p)) => p,
            Some(Err(e)) => {
              let _ = tx.error(e);
              continue;
            }
            None => {
              break;
            }
          },
        };
        if packet.port() == DISCRIMINANT {
          if packet.has_data() {
            condition = Some(packet.deserialize_generic().unwrap());
          }
          continue;
        }
        let Some(condition) = &condition else {
          held_packets.push_back(packet);
          continue;
        };
        let case = context.config.cases.iter().find(|case| case.case == *condition);

        let op = case.map_or_else(
          || {
            trace!(case = "default", op = default, "switch:case");
            &default
          },
          |case| {
            trace!(case = case.case, op = case.case_do, "switch:case");
            &case.case_do
          },
        );
        let stream = router.entry(op.clone()).or_insert_with(|| {
          let target = path_to_entity(op).unwrap(); // unwrap ok because the config has been pre-validated.
          let op_id = target.operation_id().to_owned();
          let (route_tx, route_rx) = PacketStream::new_channels();
          let link = ComponentReference::new(origin.clone(), target);
          let tx = tx.clone();
          let callback = callback.clone();
          tokio::spawn(async move {
            match callback(link, op_id, route_rx, None, None).await {
              Ok(mut call_rx) => {
                while let Some(packet) = call_rx.next().await {
                  let _ = tx.send_result(packet);
                }
              }
              Err(e) => {
                let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
              }
            };
          });
          route_tx
        });
        let _ = stream.send(packet);
      }
    });

    async move { Ok(rx) }.boxed()
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    panic!("Merge component has a dynamic signature");
  }

  fn input_names(&self, config: &Self::Config) -> Vec<String> {
    let mut context: Vec<_> = config.context.iter().map(|n| n.name.clone()).collect();
    context.push(DISCRIMINANT.to_owned());
    context
  }

  fn decode_config(data: Option<wick_packet::OperationConfig>) -> Result<Self::Config, ComponentError> {
    let config = data.ok_or_else(|| {
      ComponentError::message("Merge component requires configuration, please specify configuration.")
    })?;
    Ok(Self::Config {
      context: config.get_into("context").map_err(ComponentError::new)?,
      outputs: config.get_into("outputs").map_err(ComponentError::new)?,
      cases: config.get_into("cases").map_err(ComponentError::new)?,
      default: config.get_into("default").map_err(ComponentError::new)?,
    })
  }
}
