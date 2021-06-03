use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use crate::anyhow::Context as AnyhowContext;
use crate::connection_downstream::ConnectionDownstream;
use crate::deserialize;
use crate::dispatch::VinoEntity;
use crate::error::VinoError;
use crate::manifest::schematic_definition::{ConnectionDefinition, ConnectionTargetDefinition};
use crate::network::MetadataMap;
use crate::network::{ComponentMetadata, MapInvocation, Network};
use crate::port_entity::PortEntity;
use crate::schematic_response::{get_schematic_output, push_to_schematic_output};
use crate::MessagePayload;
use actix::prelude::*;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use vino_guest::Signal;
use wascap::prelude::KeyPair;

use super::dispatch::InvocationResponse;
use super::{
    dispatch::Invocation, schematic_response::initialize_schematic_output, SchematicDefinition,
};
use crate::Result;
type TransactionMap = HashMap<String, InputRefMap>;
type InputRefMap = HashMap<String, BufferMap>;
type BufferMap = HashMap<String, PortBuffer>;
type PortBuffer = VecDeque<MessagePayload>;

#[derive(Debug)]
enum ComponentStatus {
    Ready(Invocation),
    Waiting,
    ShortCircuit(MessagePayload),
}

pub struct Schematic {
    pub network: Option<Addr<Network>>,
    pub host_id: String,
    pub components: HashMap<String, ComponentMetadata>,
    pub recipients: HashMap<String, Recipient<Invocation>>,
    pub seed: String,
    pub transaction_map: TransactionMap,
    pub references: HashMap<String, String>,
    pub definition: SchematicDefinition,
    pub invocation_map: HashMap<String, ConnectionDownstream>,
}

impl Supervised for Schematic {}

impl Default for Schematic {
    fn default() -> Self {
        Schematic {
            network: None,
            host_id: "".to_string(),
            components: HashMap::new(),
            recipients: HashMap::new(),
            seed: "".to_string(),
            transaction_map: TransactionMap::new(),
            references: HashMap::new(),
            definition: SchematicDefinition::default(),
            invocation_map: HashMap::new(),
        }
    }
}

impl Actor for Schematic {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("Schematic started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // NOTE: do not attempt to log asynchronously in a stopped function,
        // resources (including stdout) may not be available
    }
}

impl Schematic {
    fn get_downstream_recipient(&self, name: String) -> Option<Recipient<Invocation>> {
        trace!("getting downstream recipient {}", name);
        match self.definition.components.get(&name) {
            Some(comp) => self
                .components
                .get(&comp.id)
                .map(|component| component.addr.clone()),
            None => None,
        }
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
            .definition
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
                    from: ConnectionTargetDefinition {
                        port: connection.from.port.to_string(),
                        instance: connection.from.instance.to_string(),
                    },
                    to: ConnectionTargetDefinition {
                        port: connection.to.port.to_string(),
                        instance: connection.to.instance.to_string(),
                    },
                })
            })
            .collect();
        connections
    }

    fn push_to_port(
        &mut self,
        tx_id: String,
        port: &PortEntity,
        data: MessagePayload,
    ) -> Result<ComponentStatus> {
        let reference = port.reference.to_string();

        let kp = KeyPair::from_seed(&self.seed)?;

        let refmap = self
            .transaction_map
            .entry(tx_id.to_string())
            .or_insert_with(new_refmap);

        let actor = self
            .references
            .get(&reference)
            .context(format!("Could not find reference {}", reference))?;
        trace!("pushing to {}", port);
        let key = reference.to_string();
        let metadata = self.components.get(actor).ok_or(format!(
            "Could not find metadata for {}. Component may have failed to load.",
            actor
        ))?;

        refmap
            .entry(key)
            .or_insert_with(|| new_inputbuffer_map(metadata));

        push_to_portbuffer(refmap, reference.to_string(), port.name.clone(), data)?;

        if !component_has_data(refmap, &reference) {
            return Ok(ComponentStatus::Waiting);
        }

        trace!("{} is ready to execute", reference);

        let payloads = get_component_data(refmap, &reference)?;

        let mut job_data: HashMap<String, Vec<u8>> = HashMap::new();
        for (port, payload) in payloads {
            if let MessagePayload::MessagePack(bytes) = payload {
                job_data.insert(port, bytes);
            } else {
                return Ok(ComponentStatus::ShortCircuit(payload));
            }
        }

        Ok(ComponentStatus::Ready(Invocation::next(
            &tx_id,
            &kp,
            VinoEntity::Schematic(port.schematic.to_string()),
            VinoEntity::Component(reference.to_string()),
            "job",
            MessagePayload::MultiBytes(job_data),
        )))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PassedJobArgs {
    connection: ConnectionDownstream,
    input: HashMap<String, Vec<u8>>,
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
    trace!("creating new inputbuffer map for {:?}", metadata);
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

// impl Handler<Initialize> for Schematic {
//     type Result = ResponseActFuture<Self, Result<()>>;

//     fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
//         trace!("Initializing schematic");
//         let thing = async move { Ok(()) }.into_actor(self);

//         Box::pin(thing)
//     }
// }

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) struct Initialize {
    pub host_id: String,
    pub schematic: SchematicDefinition,
    pub components: MetadataMap,
    pub seed: String,
    pub network: Addr<Network>,
}

impl Handler<Initialize> for Schematic {
    type Result = ();

    fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
        trace!("Initializing schematic {}", msg.schematic.name);
        self.seed = msg.seed;
        self.components = msg.components;
        self.host_id = msg.host_id;
        self.network = Some(msg.network);

        msg.schematic
            .components
            .iter()
            .for_each(|(instance, actor)| {
                self.references
                    .insert(instance.to_string(), actor.id.to_string());
            });

        self.definition = msg.schematic;
    }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<SchematicResponse>")]
pub struct Request {
    pub tx_id: String,
    pub schematic: String,
    pub payload: HashMap<String, Vec<u8>>,
}

#[derive(Debug)]
pub struct SchematicResponse {
    pub payload: Vec<u8>,
}

impl Handler<Request> for Schematic {
    type Result = ResponseActFuture<Self, Result<SchematicResponse>>;

    fn handle(&mut self, msg: Request, ctx: &mut Context<Self>) -> Self::Result {
        trace!("Requesting schematic '{}'", msg.schematic);
        let tx_id = msg.tx_id.to_string();
        let schematic = msg.schematic.to_string();

        let invocations = gen_packets(self, tx_id.to_string(), schematic.to_string(), msg.payload);

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

impl Handler<ResponseFuture> for Schematic {
    type Result = ResponseActFuture<Self, Result<InvocationResponse>>;

    fn handle(&mut self, msg: ResponseFuture, _ctx: &mut Context<Self>) -> Self::Result {
        trace!(
            "Requesting future for schematic '{}' on tx {}",
            msg.schematic,
            msg.tx_id
        );

        let tx_id = msg.tx_id;

        let schematic_name = msg.schematic;
        let timeout = Duration::from_millis(1000);
        let schematic = get_schematic_output(&tx_id, &schematic_name);

        let task = async move {
            match schematic {
                Ok(future) => match actix_rt::time::timeout(timeout, future).await {
                    Ok(r) => Ok(r),
                    Err(e) => Ok(InvocationResponse::error(
                        tx_id,
                        format!("Error waiting for schematic output {}", e.to_string()),
                    )),
                },
                Err(e) => Ok(InvocationResponse::error(tx_id, e.to_string())),
            }
        }
        .into_actor(self);

        Box::pin(task)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<()>")]
struct MessagePacket {
    tx_id: String,
    origin: VinoEntity,
    target: PortEntity,
    payload: MessagePayload,
}

fn gen_packets(
    sm: &mut Schematic,
    tx_id: String,
    name: String,
    bytemap: HashMap<String, Vec<u8>>,
) -> Result<Vec<MessagePacket>> {
    let schematic = &sm.definition;
    let _kp = KeyPair::from_seed(&sm.seed).context("Couldn't create keypair")?;

    let schematic_outputs = schematic.get_output_names();

    initialize_schematic_output(&tx_id, &name, schematic_outputs);

    let invocations: Vec<MessagePacket> = schematic
        .connections
        .iter()
        .filter(|conn| conn.from.instance == crate::SCHEMATIC_INPUT)
        .map(|conn| {
            let bytes = bytemap
                .get(&conn.from.port)
                .unwrap_or_else(|| panic!("Output on port '{}' not found", conn.to.instance));

            MessagePacket {
                target: PortEntity {
                    schematic: name.to_string(),
                    name: conn.to.port.to_string(),
                    reference: conn.to.instance.to_string(),
                },
                origin: VinoEntity::Schematic(name.to_string()),
                tx_id: tx_id.to_string(),
                payload: MessagePayload::MessagePack(bytes.clone()),
            }
        })
        .collect();
    Ok(invocations)
}

async fn handle_schematic_invocation(
    invocations: Vec<MessagePacket>,
    schematic: Addr<Schematic>,
    tx_id: String,
    target: String,
) -> Result<InvocationResponse> {
    let invocations = try_join_all(invocations.into_iter().map(|inv| schematic.send(inv)));

    invocations
        .await
        .map_err(|e| format!("Error pushing to schematic ports: {}", e))?;

    let response = schematic
        .send(ResponseFuture {
            tx_id: tx_id.to_string(),
            schematic: target,
        })
        .await
        .map_err(|e| format!("Error pushing to schematic ports: {}", e))??;

    Ok(response)
}

impl Handler<MessagePacket> for Schematic {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: MessagePacket, ctx: &mut Context<Self>) -> Self::Result {
        let name = self.definition.name.to_string();
        let port = msg.target;
        let payload = msg.payload;
        let tx_id = msg.tx_id;
        trace!("Receiving on port {}", port);

        let reference = port.reference.to_string();
        //TODO normalize output to the same buffers as regular ports
        if reference == crate::SCHEMATIC_OUTPUT {
            return Box::pin(
                async move {
                    push_to_schematic_output(&tx_id, &name, &port.name, payload)?;
                    Ok(())
                }
                .into_actor(self),
            );
        }
        let status = self.push_to_port(tx_id.to_string(), &port, payload);
        let schematic_host = ctx.address();

        let receiver = self.get_downstream_recipient(reference.to_string());
        let network = self.network.clone().unwrap();

        Box::pin(
            async move {
                match status {
                    Err(err) => {
                        let e = format!("Error handling IP: {}", err);
                        error!("{}", e);
                        Err(e.into())
                    }
                    Ok(ComponentStatus::ShortCircuit(payload)) => match schematic_host
                        .send(ShortCircuit {
                            tx_id: tx_id.to_string(),
                            schematic: name,
                            reference,
                            payload,
                        })
                        .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Error deserializing job signal: {}", e).into()),
                    },

                    Ok(ComponentStatus::Waiting) => {
                        trace!("Component {} is still waiting on data", reference);
                        Ok(())
                    }
                    Ok(ComponentStatus::Ready(invocation)) => {
                        network.do_send(MapInvocation {
                            inv_id: invocation.id.to_string(),
                            tx_id: tx_id.clone(),
                            schematic: name.to_string(),
                            entity: invocation.target.clone(),
                        });

                        let response: std::result::Result<Signal, _> = match receiver {
                            Some(receiver) => match receiver.send(invocation).await {
                                Ok(response) => deserialize(&response.msg),
                                Err(err) => Err(format!("Error executing job: {}", err).into()),
                            },
                            None => Err(anyhow!("No receiver found").into()),
                        };

                        match response {
                            Ok(_signal) => Ok(()),
                            Err(err) => {
                                warn!(
                                    "Tx '{}': schematic '{}' short-circuiting from '{}': {}",
                                    tx_id,
                                    name,
                                    reference,
                                    err.to_string()
                                );
                                schematic_host
                                    .send(ShortCircuit {
                                        tx_id: tx_id.to_string(),
                                        schematic: name.to_string(),
                                        reference: reference.to_string(),
                                        payload: MessagePayload::Error(err.to_string()),
                                    })
                                    .await
                                    .map(|_| ())
                                    .map_err(|e| e.to_string().into())
                            }
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
pub struct ShortCircuit {
    pub tx_id: String,
    pub schematic: String,
    pub reference: String,
    pub payload: MessagePayload,
}

impl Handler<ShortCircuit> for Schematic {
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
        trace!(
            "Connections to short {:?}",
            ConnectionDefinition::print_all(&downstreams)
        );
        let outputs: Vec<OutputReady> = downstreams
            .iter()
            .map(|conn| OutputReady {
                tx_id: tx_id.to_string(),
                port: PortEntity {
                    reference: conn.from.instance.to_string(),
                    name: conn.from.port.to_string(),
                    schematic: schematic.to_string(),
                },
                payload: payload.clone(),
            })
            .collect();
        let schematic_host = ctx.address();

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

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct OutputReady {
    pub port: PortEntity,
    pub tx_id: String,
    pub payload: MessagePayload,
}

impl Handler<OutputReady> for Schematic {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: OutputReady, ctx: &mut Context<Self>) -> Self::Result {
        trace!("Output ready on {}", msg.port);
        let seed = self.seed.to_string();

        let reference = msg.port.reference;
        let port = msg.port.name;
        let tx_id = msg.tx_id;

        let data = msg.payload;
        let schematic = msg.port.schematic;

        let defs = self.get_connections(reference.to_string(), port.to_string());
        let addr = ctx.address();
        let task = async move {
            let _kp = KeyPair::from_seed(&seed).unwrap();
            let origin = VinoEntity::Port(PortEntity {
                schematic: schematic.to_string(),
                name: port.to_string(),
                reference: reference.to_string(),
            });
            let make_packet = |conn: ConnectionDefinition| {
                let entity = PortEntity {
                    schematic: schematic.to_string(),
                    name: conn.to.port.to_string(),
                    reference: conn.to.instance,
                };
                MessagePacket {
                    tx_id: tx_id.clone(),
                    origin: origin.clone(),
                    target: entity,
                    payload: data.clone(),
                }
            };
            let invocations =
                try_join_all(defs.into_iter().map(make_packet).map(|ips| addr.send(ips)));
            invocations.await?;
            Ok::<(), VinoError>(())
        }
        .into_actor(self);

        Box::pin(task)
    }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<InvocationResponse>")]
pub struct ResponseFuture {
    pub schematic: String,
    pub tx_id: String,
}

#[cfg(test)]
mod test {

    use crate::{
        components::wapc_component_actor::WapcComponentActor,
        manifest::schematic_definition::{
            ComponentDefinition, ConnectionDefinition, ConnectionTargetDefinition,
        },
        network::ActorPorts,
        network::ComponentMetadata,
        util::hlreg::HostLocalSystemService,
    };

    use super::*;

    #[test_env_log::test(actix_rt::test)]
    async fn test_init() -> crate::Result<()> {
        trace!("test_init");
        // let actor = WapcComponentActor::default();
        let component_addr = SyncArbiter::start(1, WapcComponentActor::default);

        // let component_addr = actor.start();
        let schematic = Schematic::default();
        let addr = schematic.start();
        let mut schem_def = SchematicDefinition::default();
        schem_def.components.insert(
            "logger".to_string(),
            ComponentDefinition {
                metadata: None,
                id: "vino::log".to_string(),
            },
        );
        schem_def.connections.push(ConnectionDefinition {
            from: ConnectionTargetDefinition {
                instance: "vino::schematic".to_string(),
                port: "input".to_string(),
            },
            to: ConnectionTargetDefinition {
                instance: "logger".to_string(),
                port: "input".to_string(),
            },
        });
        schem_def.connections.push(ConnectionDefinition {
            from: ConnectionTargetDefinition {
                instance: "logger".to_string(),
                port: "output".to_string(),
            },
            to: ConnectionTargetDefinition {
                instance: "vino::schematic".to_string(),
                port: "output".to_string(),
            },
        });
        let mut refs = MetadataMap::new();
        refs.insert(
            "logger".to_string(),
            ComponentMetadata {
                ports: ActorPorts {
                    inputs: vec!["input".to_string()],
                    outputs: vec!["output".to_string()],
                },
                addr: component_addr.recipient(),
            },
        );
        let hostkey = KeyPair::new_server();

        addr.send(Initialize {
            network: Network::from_hostlocal_registry(&KeyPair::new_server().public_key()),
            host_id: "test".to_string(),
            schematic: schem_def,
            components: refs,
            seed: hostkey.seed()?,
        })
        .await?;
        let mut input: HashMap<String, Vec<u8>> = HashMap::new();
        input.insert("input".to_string(), vec![20]);
        let response = addr
            .send(super::Request {
                tx_id: "rsat".to_string(),
                schematic: "logger".to_string(),
                payload: input,
            })
            .await?;
        println!("{:?}", response);
        // let futures = vec![];
        // try_join_all(futures).await?;
        Ok(())
    }
}
