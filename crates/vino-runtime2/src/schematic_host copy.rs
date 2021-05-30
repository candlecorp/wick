use crate::dispatch::{Invocation, MessagePayload, VinoEntity};
use crate::hlreg::HostLocalSystemService;
use crate::InvocationResponse;
use crate::Result;

use actix::prelude::*;
use anyhow::Context as AnyContext;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    time::Duration,
};
use vino_guest::{OutputPayload, Signal};
use wascap::prelude::KeyPair;

use super::{
    messages::RequestResponse,
    messages::ShortCircuit,
    port_entity::PortEntity,
    schematic_response::push_to_schematic_output,
    schematic_response::{get_schematic_output, initialize_schematic_output},
    serdes::{deserialize, serialize},
    HasSchematic, Initialize, OutputReady, RegisterReference, RegisterSchematic,
    SchematicDefinition,
};

type TransactionMap = HashMap<String, InputRefMap>;
type InputRefMap = HashMap<String, BufferMap>;
type BufferMap = HashMap<String, PortBuffer>;
type PortBuffer = VecDeque<MessagePayload>;

#[derive(Debug, Serialize, Deserialize)]
struct PassedJobArgs {
    connection: ConnectionDownstream,
    input: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Default)]
pub struct ComponentMetadata {
    ports: super::ActorPorts,
}
pub struct SchematicHost {
    components: HashMap<String, ComponentMetadata>,
    host_id: String,
    seed: String,
    transaction_map: TransactionMap,
    schematics: HashMap<String, SchematicDefinition>,
    references: HashMap<String, String>,
}

impl Default for SchematicHost {
    fn default() -> Self {
        SchematicHost {
            components: HashMap::new(),
            host_id: "".to_string(),
            seed: "".to_string(),
            transaction_map: HashMap::new(),
            schematics: HashMap::new(),
            references: HashMap::new(),
        }
    }
}

impl Supervised for SchematicHost {}

impl SystemService for SchematicHost {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        trace!("Schematic host started");
        ctx.set_mailbox_capacity(1000);
    }
}

impl HostLocalSystemService for SchematicHost {}

impl Actor for SchematicHost {
    type Context = Context<Self>;
}

fn error_maker<T: AsRef<str>>(tx_id: T, inv_id: T) -> impl FnOnce(T) -> InvocationResponse {
    move |e: T| InvocationResponse::transaction_error(tx_id.as_ref(), inv_id.as_ref(), e.as_ref())
}

impl SchematicHost {
    fn bus(&self) -> Addr<MessageBus> {
        MessageBus::from_hostlocal_registry(&self.host_id)
    }
    fn has_schematic_locally(&self, schematic: &str) -> bool {
        self.schematics.contains_key(schematic)
    }
    fn get_outputs(&self, reference: String) -> Vec<String> {
        match self.references.get(&reference) {
            Some(actor) => match self.components.get(actor) {
                Some(metadata) => metadata.ports.outputs.clone(),
                None => vec![],
            },
            None => vec![],
        }
    }
    fn get_connections(&self, reference: String, port: String) -> Vec<ConnectionDefinition> {
        let references = &self.references;
        let connections: Vec<ConnectionDefinition> = self
            .schematics
            .values()
            .flat_map::<Vec<ConnectionDefinition>, _>(|schematic| {
                schematic
                    .connections
                    .iter()
                    .filter(|connection| {
                        connection.from.instance == reference && connection.from.port == port
                    })
                    .filter_map(|connection| {
                        let from_actor = match references.get(&connection.from.instance) {
                            Some(a) => Some(a),
                            None => {
                                if connection.from.instance == crate::SCHEMATIC_INPUT {
                                    Some(&connection.from.instance)
                                } else {
                                    return None;
                                }
                            }
                        };
                        let to_actor = match references.get(&connection.to.instance) {
                            Some(a) => Some(a),
                            None => {
                                if connection.to.instance == crate::SCHEMATIC_OUTPUT {
                                    Some(&connection.to.instance)
                                } else {
                                    return None;
                                }
                            }
                        };
                        if from_actor.is_none() || to_actor.is_none() {
                            return None;
                        }
                        Some(ConnectionDefinition {
                            from: ConnectionTarget {
                                port: connection.from.port.to_string(),
                                reference: connection.from.instance.to_string(),
                                actor: from_actor.unwrap().to_string(),
                            },
                            to: ConnectionTarget {
                                port: connection.to.port.to_string(),
                                reference: connection.to.instance.to_string(),
                                actor: to_actor.unwrap().to_string(),
                            },
                        })
                    })
                    .collect()
            })
            .collect();
        connections
    }

    fn push_to_port(
        &mut self,
        tx_id: String,
        port: PortEntity,
        data: MessagePayload,
    ) -> Result<ComponentStatus> {
        let reference = port.reference.to_string();

        let kp = KeyPair::from_seed(&self.seed)?;

        let refmap = self
            .transaction_map
            .entry(tx_id.to_string())
            .or_insert_with(new_refmap);

        let actor = port.parent.to_string();
        let key = reference.to_string();
        let metadata = self
            .components
            .get(&actor)
            .ok_or("Could not get component metadata")?;

        refmap
            .entry(key)
            .or_insert_with(|| new_inputbuffer_map(metadata));

        push_to_portbuffer(refmap, reference.to_string(), port.name.clone(), data)?;

        if !component_has_data(refmap, &reference) {
            return Ok(ComponentStatus::Waiting);
        }

        trace!("{} is ready to execute", reference);

        let payloads = get_component_data(refmap, &reference)?;

        let downstream = ConnectionDownstream::new(
            self.host_id.to_string(),
            port.schematic.to_string(),
            tx_id.to_string(),
            port.parent.to_string(),
            reference,
        );

        let mut job_data: HashMap<String, Vec<u8>> = HashMap::new();
        for (port, payload) in payloads {
            if let MessagePayload::Bytes(bytes) = payload {
                job_data.insert(port, bytes);
            } else {
                return Ok(ComponentStatus::ShortCircuit(payload));
            }
        }

        let job_args = PassedJobArgs {
            connection: downstream,
            input: job_data,
        };
        let serialized_data = serialize(&job_args)?;

        Ok(ComponentStatus::Ready(Invocation::next(
            &tx_id,
            &kp,
            WasmCloudEntity::InputPort(port.clone()),
            WasmCloudEntity::Actor(port.parent),
            "job",
            serialized_data,
        )))
    }
}

impl Handler<HasSchematic> for SchematicHost {
    type Result = ResponseActFuture<Self, bool>;

    fn handle(&mut self, msg: HasSchematic, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("querying for schematic {}", msg.schematic);

        let result = self.has_schematic_locally(&msg.schematic);

        Box::pin(async move { result }.into_actor(self))
    }
}

impl Handler<ShortCircuit> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: ShortCircuit, ctx: &mut Context<Self>) -> Self::Result {
        trace!("Short circuiting component {}", msg.reference);
        let reference = msg.reference;
        let tx_id = msg.tx_id;
        let schematic = msg.schematic;
        let payload = msg.payload;

        let outputs = self.get_outputs(reference.to_string());
        let downstreams: Vec<ConnectionDefinition> = outputs
            .iter()
            .flat_map(|port| self.get_connections(reference.to_string(), port.to_string()))
            .collect();
        trace!("Connections to short {:?}", downstreams);
        let outputs: Vec<OutputReady> = downstreams
            .iter()
            .map(|conn| OutputReady {
                tx_id: tx_id.to_string(),
                port: PortEntity {
                    reference: conn.from.reference.to_string(),
                    name: conn.from.port.to_string(),
                    parent: conn.from.actor.to_string(),
                    schematic: schematic.to_string(),
                },
                payload: payload.clone(),
            })
            .collect();
        let schematic_host = ctx.address();
        trace!("Short circuiting downstreams {:?}", outputs);
        let futures = outputs
            .into_iter()
            .map(move |message| schematic_host.send(message));

        Box::pin(
            async move {
                match try_join_all(futures).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.into()),
                }
            }
            .into_actor(self),
        )
    }
}

impl Handler<RequestResponse> for SchematicHost {
    type Result = ResponseActFuture<Self, InvocationResponse>;

    fn handle(&mut self, msg: RequestResponse, _ctx: &mut Context<Self>) -> Self::Result {
        let tx_id = msg.tx_id;

        let schematic_name = msg.schematic;
        let timeout = Duration::from_millis(1000);
        let schematic = get_schematic_output(&tx_id, &schematic_name);

        let task = async move {
            match schematic {
                Ok(future) => match actix_rt::time::timeout(timeout, future).await {
                    Ok(r) => r,
                    Err(e) => InvocationResponse::transaction_error(
                        &tx_id,
                        &tx_id,
                        &format!("Error waiting for schematic output {}", e.to_string()),
                    ),
                },
                Err(e) => InvocationResponse::transaction_error(&tx_id, &tx_id, &e.to_string()),
            }
        }
        .into_actor(self);

        Box::pin(task)
    }
}

impl Handler<Initialize> for SchematicHost {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Schematic host initializing on {}", msg.host_id);
        self.host_id = msg.host_id;
        self.seed = msg.seed;

        Box::pin(async {}.into_actor(self))
    }
}

impl Handler<RegisterSchematic> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: RegisterSchematic, ctx: &mut Context<Self>) -> Self::Result {
        trace!("Registering Schematic");
        let mb = self.bus();
        let schematic = msg.schematic;
        let name = schematic.name.to_string();
        schematic.components.iter().for_each(|(instance, actor)| {
            self.references
                .insert(instance.to_string(), actor.actor_ref.to_string());
        });
        self.schematics.insert(name.clone(), schematic);

        let recipient = ctx.address().recipient();
        Box::pin(
            async move {
                let entity = WasmCloudEntity::Schematic(name);
                let sub = Subscribe {
                    interest: entity,
                    subscriber: recipient,
                };
                mb.send(sub).await?;
                Ok(())
            }
            .into_actor(self),
        )
    }
}

impl Handler<OutputReady> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: OutputReady, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Output ready");
        let mb = self.bus();
        let seed = self.seed.to_string();

        let reference = msg.port.reference;
        let port = msg.port.name;
        let tx_id = msg.tx_id;
        let actor = msg.port.parent;
        let data = msg.payload;
        let schematic = msg.port.schematic;

        let defs = self.get_connections(reference.to_string(), port.to_string());

        let task = async move {
            let kp = KeyPair::from_seed(&seed).unwrap();
            let origin = WasmCloudEntity::OutputPort(PortEntity {
                schematic: schematic.to_string(),
                name: port.to_string(),
                reference: reference.to_string(),
                parent: actor.to_string(),
            });
            let make_invocation = |conn: ConnectionDefinition| {
                let (entity, op) = if conn.to.reference == crate::SCHEMATIC_OUTPUT {
                    (
                        WasmCloudEntity::Schematic(schematic.to_string()),
                        conn.to.port,
                    )
                } else {
                    (
                        WasmCloudEntity::InputPort(PortEntity {
                            schematic: schematic.to_string(),
                            name: conn.to.port.to_string(),
                            reference: conn.to.reference.to_string(),
                            parent: conn.to.actor,
                        }),
                        "".to_string(),
                    )
                };
                Invocation::next(&tx_id, &kp, origin.clone(), entity, &op, data.clone())
            };
            let invocations = try_join_all(
                defs.into_iter()
                    .map(make_invocation)
                    .map(|invocation| mb.send(invocation)),
            );
            invocations.await?;
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        }
        .into_actor(self);

        Box::pin(task)
    }
}

impl Handler<RegisterReference> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: RegisterReference, ctx: &mut Context<Self>) -> Self::Result {
        trace!("Register component {} with connection manager", msg.id);
        let mb = self.bus();
        let namespace = msg.namespace.to_string();
        let actor_id = msg.id.to_string();
        let reference = msg.reference.to_string();
        let make_entity = |port: &String| PortEntity {
            schematic: namespace.to_string(),
            reference: reference.to_string(),
            parent: actor_id.to_string(),
            name: port.to_string(),
        };

        let entities: Vec<WasmCloudEntity> = itertools::concat(vec![
            msg.ports
                .inputs
                .iter()
                .map(|port| WasmCloudEntity::InputPort(make_entity(port)))
                .collect(),
            msg.ports
                .outputs
                .iter()
                .map(|port| WasmCloudEntity::OutputPort(make_entity(port)))
                .collect(),
        ]);
        self.components
            .insert(msg.id, ComponentMetadata { ports: msg.ports });

        let recipient = ctx.address().recipient();
        Box::pin(
            async move {
                let sub_requests = entities.iter().map(|entity| {
                    mb.send(Subscribe {
                        interest: entity.clone(),
                        subscriber: recipient.clone(),
                    })
                });
                match try_join_all(sub_requests).await {
                    Ok(_) => {
                        trace!("Component subscriptions registered");
                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                }
            }
            .into_actor(self),
        )
    }
}

async fn handle_schematic_invocation(
    invocations: Vec<Invocation>,
    mb: Addr<SchematicHost>,
    tx_id: String,
    target: String,
) -> Result<InvocationResponse> {
    let invocations = try_join_all(invocations.into_iter().map(|inv| mb.send(inv)));

    invocations
        .await
        .map_err(|e| format!("Error pushing to schematic ports: {}", e))?;
    let response = mb
        .send(RequestResponse {
            tx_id: tx_id.to_string(),
            schematic: target,
        })
        .await
        .map_err(|e| format!("Error pushing to schematic ports: {}", e))?;

    Ok(response)
}

#[derive(Message, Clone)]
#[rtype(result = "Result<()>")]
pub struct SchematicOutputReceived {
    pub tx_id: String,
    pub inv_id: String,
    pub schematic: String,
    pub payload: MessagePayload,
    pub port: String,
}
impl Handler<SchematicOutputReceived> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: SchematicOutputReceived, _ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(
            async move {
                match push_to_schematic_output(&msg.tx_id, &msg.schematic, &msg.port, msg.payload) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            .into_actor(self),
        )
    }
}

#[derive(Message, Clone)]
#[rtype(result = "Result<SchematicResponse>")]
pub struct SchematicRequest {
    pub tx_id: String,
    pub schematic: String,
    pub payload: HashMap<String, Vec<u8>>,
}

pub struct SchematicResponse {
    pub payload: Vec<u8>,
}
impl Handler<SchematicRequest> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<SchematicResponse>>;

    fn handle(&mut self, msg: SchematicRequest, ctx: &mut Context<Self>) -> Self::Result {
        let tx_id = msg.tx_id.to_string();
        let schematic = msg.schematic.to_string();

        let invocations = generate_multiport_invocations(
            self,
            tx_id.to_string(),
            schematic.to_string(),
            msg.payload,
        );

        let host = ctx.address();

        Box::pin(
            async move {
                match invocations {
                    Ok(invocations) => {
                        match handle_schematic_invocation(invocations, host, tx_id, schematic).await
                        {
                            Ok(a) => Ok(SchematicResponse { payload: a.msg }),
                            Err(e) => Err(e.to_string().into()),
                        }
                    }
                    Err(e) => Err(e.to_string().into()),
                }
            }
            .into_actor(self),
        )
    }
}

#[derive(Message, Clone)]
#[rtype(result = "Result<()>")]
pub struct InputReceived {
    pub tx_id: String,
    pub inv_id: String,
    pub port_entity: PortEntity,
    pub payload: MessagePayload,
}

impl Handler<InputReceived> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: InputReceived, ctx: &mut Context<Self>) -> Self::Result {
        let port = msg.port_entity;
        let payload = msg.payload;
        let tx_id = msg.tx_id;
        let inv_id = msg.inv_id;

        trace!("Receiving on port {}:{}", port.reference, port.name);
        let schematic = port.schematic.to_string();
        let reference = port.reference.to_string();
        let status = self.push_to_port(tx_id.to_string(), port, payload);
        let mb = self.bus();
        let schematic_host = ctx.address();
        Box::pin(
            async move {
                match status {
                    Err(err) => Err(format!("Error executing job: {}", err).into()),
                    Ok(ComponentStatus::ShortCircuit(payload)) => match schematic_host
                        .send(ShortCircuit {
                            inv_id: inv_id.to_string(),
                            tx_id: tx_id.to_string(),
                            schematic,
                            reference,
                            payload,
                        })
                        .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Error deserializing job signal: {}", e).into()),
                    },

                    Ok(ComponentStatus::Waiting) => Ok(()),
                    Ok(ComponentStatus::Ready(invocation)) => {
                        let response = match mb.send(invocation).await {
                            Ok(response) => deserialize(&response.msg),
                            Err(err) => Err(format!("Error executing job: {}", err).into()),
                        };

                        match response {
                            Ok(Signal::Done) => Ok(()),
                            Err(err) => schematic_host
                                .send(ShortCircuit {
                                    tx_id: tx_id.to_string(),
                                    inv_id: inv_id.to_string(),
                                    schematic: schematic.to_string(),
                                    reference: reference.to_string(),
                                    payload: MessagePayload::Error(err.to_string()),
                                })
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string().into()),
                        }
                    }
                }
            }
            .into_actor(self),
        )
    }
}

#[derive(Message, Clone)]
#[rtype(result = "Result<()>")]
pub struct OutputReceived {
    pub tx_id: String,
    pub inv_id: String,
    pub port_entity: PortEntity,
    pub payload: MessagePayload,
}

impl Handler<OutputReceived> for SchematicHost {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: OutputReceived, ctx: &mut Context<Self>) -> Self::Result {
        let port = msg.port_entity;
        let payload = msg.payload;
        let tx_id = msg.tx_id;
        trace!("Pushing out of port {}:{}", port.reference, port.name);
        let sender = ctx.address();

        let message_payload = match payload {
            MessagePayload::Bytes(bytes) => {
                let data = deserialize::<OutputPayload>(&bytes)
                    .map_err(|e| MessagePayload::Error(e.to_string()));

                match data {
                    Ok(payload) => match payload {
                        OutputPayload::Bytes(b) => MessagePayload::Bytes(b),
                        OutputPayload::Exception(e) => MessagePayload::Exception(e),
                        OutputPayload::Error(e) => MessagePayload::Error(e),
                    },
                    Err(e) => e,
                }
            }
            MessagePayload::MultiBytes(_) => {
                MessagePayload::Error("Invalid payload type sent to output port".to_string())
            }
            mp => mp,
        };

        Box::pin(
            async move {
                match sender
                    .send(OutputReady {
                        port,
                        payload: message_payload,
                        tx_id: tx_id.to_string(),
                    })
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(err) => Err(format!("Error executing job: {}", err).into()),
                }
            }
            .into_actor(self),
        )
    }
}

impl Handler<Invocation> for SchematicHost {
    type Result = ResponseActFuture<Self, InvocationResponse>;

    fn handle(&mut self, msg: Invocation, ctx: &mut Context<Self>) -> Self::Result {
        let tx_id = msg.tx_id.to_string();
        let inv_id = msg.id.to_string();
        let operation = msg.operation.to_string();
        trace!(
            "Schematic host: {} invoking entity {}",
            msg.origin_url(),
            msg.target_url()
        );

        let tx_id_bytes = serialize(&tx_id).unwrap_or_else(|_e| vec![]);
        let target = msg.target;
        let data = msg.msg;
        let schematic_host = ctx.address();

        let success = InvocationResponse::transaction_success(&tx_id, &inv_id, &tx_id_bytes);
        let make_error = error_maker(tx_id.to_string(), inv_id.to_string());

        Box::pin(
            async move {
                match (target, data) {
                    (WasmCloudEntity::Schematic(name), MessagePayload::MultiBytes(bytemap)) => {
                        match schematic_host
                            .send(SchematicRequest {
                                tx_id: tx_id.to_string(),
                                schematic: name,
                                payload: bytemap,
                            })
                            .await
                        {
                            Ok(Ok(response)) => InvocationResponse::transaction_success(
                                &tx_id,
                                &inv_id,
                                response.payload,
                            ),
                            Ok(Err(e)) => make_error(e.to_string()),
                            Err(e) => make_error(e.to_string()),
                        }
                    }
                    (WasmCloudEntity::Schematic(name), payload) => match schematic_host
                        .send(SchematicOutputReceived {
                            tx_id,
                            inv_id,
                            schematic: name,
                            payload,
                            port: operation,
                        })
                        .await
                    {
                        Ok(_) => success,
                        Err(e) => make_error(e.to_string()),
                    },
                    (WasmCloudEntity::InputPort(port), payload) => match schematic_host
                        .send(InputReceived {
                            tx_id,
                            inv_id,
                            port_entity: port,
                            payload,
                        })
                        .await
                    {
                        Ok(_) => success,
                        Err(e) => make_error(e.to_string()),
                    },
                    (WasmCloudEntity::OutputPort(port), payload) => match schematic_host
                        .send(OutputReceived {
                            tx_id,
                            inv_id,
                            port_entity: port,
                            payload,
                        })
                        .await
                    {
                        Ok(_) => success,
                        Err(e) => make_error(e.to_string()),
                    },
                    _ => make_error(
                        "Schematic received invocation on invalid configuration".to_string(),
                    ),
                }
            }
            .into_actor(self),
        )
    }
}

fn generate_multiport_invocations(
    sm: &mut SchematicHost,
    tx_id: String,
    name: String,
    bytemap: HashMap<String, Vec<u8>>,
) -> Result<Vec<Invocation>> {
    let schematic = sm.schematics.get(&name).context("Schematic not found")?;
    let schematic = schematic.clone();
    let kp = KeyPair::from_seed(&sm.seed).context("Couldn't create keypair")?;

    let schematic_outputs = schematic.get_output_names();

    initialize_schematic_output(&tx_id, &name, schematic_outputs);

    let invocations: Vec<Invocation> = schematic
        .connections
        .iter()
        .filter(|conn| conn.from.instance == crate::SCHEMATIC_INPUT)
        .map(|conn| {
            // TODO need to handle this better
            let to_parent = schematic
                .components
                .get(&conn.to.instance)
                .unwrap_or_else(|| panic!("Reference '{}' not found", conn.to.instance));
            let bytes = bytemap
                .get(&conn.from.port)
                .unwrap_or_else(|| panic!("Port '{}' not found", conn.to.instance));

            Invocation::next(
                &tx_id,
                &kp,
                WasmCloudEntity::Schematic(name.to_string()),
                WasmCloudEntity::InputPort(PortEntity {
                    schematic: name.to_string(),
                    name: conn.to.port.to_string(),
                    parent: to_parent.actor_ref.to_string(),
                    reference: conn.to.instance.to_string(),
                }),
                "",
                bytes,
            )
        })
        .collect();
    Ok(invocations)
}

enum ComponentStatus {
    Ready(Invocation),
    Waiting,
    ShortCircuit(MessagePayload),
}

fn buffer_has_data(buffer: &PortBuffer) -> bool {
    !buffer.is_empty()
}

fn component_has_data(componentref_map: &InputRefMap, reference: &str) -> bool {
    match componentref_map.get(reference) {
        Some(portbuffer_map) => portbuffer_map
            .values()
            .map(|port| buffer_has_data(port))
            .all(|has_data| has_data),
        None => false,
    }
}

fn get_component_data(
    componentref_map: &mut InputRefMap,
    reference: &str,
) -> std::result::Result<HashMap<String, MessagePayload>, &'static str> {
    match componentref_map.get_mut(reference) {
        Some(portbuffer_map) => {
            let mut next: HashMap<String, MessagePayload> = HashMap::new();
            for (key, buffer) in portbuffer_map.iter_mut() {
                if let Some(value) = buffer.pop_front() {
                    next.insert(key.to_string(), value);
                } else {
                    return Err("Buffer not actually ready");
                }
            }
            Ok(next)
        }
        None => Err("Could not get buffer map"),
    }
}

fn new_refmap() -> InputRefMap {
    InputRefMap::new()
}

fn new_inputbuffer_map(metadata: &ComponentMetadata) -> BufferMap {
    metadata
        .ports
        .inputs
        .iter()
        .map(|p| (p.to_string(), VecDeque::new()))
        .collect()
}

fn push_to_portbuffer(
    component_ref_map: &mut InputRefMap,
    ref_id: String,
    port: String,
    data: MessagePayload,
) -> Result<()> {
    match component_ref_map.get_mut(&ref_id) {
        Some(portbuffer_map) => {
            trace!("Getting portbuffer for port {:?}", port);
            match portbuffer_map.get_mut(&port) {
                Some(buffer) => {
                    buffer.push_back(data);
                    Ok(())
                }
                None => {
                    Err(format!("Invalid actor state: no portbuffer for port {:?}", port).into())
                }
            }
        }
        None => Err(format!("Could not get portbuffer map for reference {}", ref_id).into()),
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ConnectionDownstream {
    pub host_id: String,
    pub namespace: String,
    pub tx_id: String,
    pub actor: String,
    pub reference: String,
}

impl ConnectionDownstream {
    pub fn new(
        host_id: String,
        namespace: String,
        tx_id: String,
        actor: String,
        reference: String,
    ) -> Self {
        ConnectionDownstream {
            host_id,
            namespace,
            tx_id,
            actor,
            reference,
        }
    }
    pub fn send(&self, port: String, data: Vec<u8>) -> Result<()> {
        let cm = SchematicHost::from_hostlocal_registry(&self.host_id);
        trace!(
            "sending to output {}[{}] (txid: {})",
            self.reference,
            port,
            self.tx_id,
        );
        let payload = match deserialize::<OutputPayload>(&data)? {
            OutputPayload::Bytes(b) => MessagePayload::Bytes(b),
            OutputPayload::Exception(e) => MessagePayload::Exception(e),
            OutputPayload::Error(e) => MessagePayload::Error(e),
        };

        cm.do_send(OutputReady {
            port: PortEntity {
                name: port,
                parent: self.actor.to_string(),
                reference: self.reference.to_string(),
                schematic: self.namespace.to_string(),
            },
            tx_id: self.tx_id.to_string(),
            payload,
        });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::generated::core::deserialize;

    use super::*;

    #[test_env_log::test(actix_rt::test)]
    async fn args_serialization() -> crate::Result<()> {
        #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
        pub struct JobEncoded {
            #[serde(rename = "input")]
            pub input: InputEncoded,
        }
        #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
        pub struct InputEncoded {
            #[serde(rename = "sample_input_a")]
            pub sample_input_a: Vec<u8>,
        }

        fn deserialize_inputs(
            args: InputEncoded,
        ) -> std::result::Result<
            Inputs,
            std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>,
        > {
            Ok(Inputs {
                sample_input_a: deserialize(&args.sample_input_a)?,
            })
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
        pub struct JobArgs {
            #[serde(rename = "input")]
            pub input: Inputs,
        }
        #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
        pub struct Inputs {
            #[serde(rename = "sample_input_a")]
            pub sample_input_a: String,
        }

        let mut job_data: HashMap<String, Vec<u8>> = HashMap::new();
        let original_input_port_a = "sample_input_a".to_string();
        let original_data = "hello world";
        job_data.insert(original_input_port_a, serialize(original_data).unwrap());

        let job_args = PassedJobArgs {
            connection: ConnectionDownstream::default(),
            input: job_data,
        };
        debug!("passed job args {:?}", job_args);
        let serialized_data = serialize(job_args).unwrap();
        debug!("serialized_data {:?}", serialized_data);

        let roundtrip: PassedJobArgs = deserialize(&serialized_data).unwrap();
        debug!("roundtrip {:?}", roundtrip);

        let deserialized_partial: JobEncoded = deserialize(&serialized_data).unwrap();
        debug!("deserialized_partial {:?}", deserialized_partial);
        let deserialized_inputs: Inputs = deserialize_inputs(deserialized_partial.input).unwrap();
        debug!("deserialized_fully {:?}", deserialized_inputs);

        assert_eq!(original_data, deserialized_inputs.sample_input_a);

        Ok(())
    }
}
