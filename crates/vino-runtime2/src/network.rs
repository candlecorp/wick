use std::collections::HashMap;
use std::fmt::Display;

use super::schematic::Schematic;
use crate::deserialize;
use crate::dispatch::MessagePayload;
use crate::hlreg::HostLocalSystemService;
use crate::manifest::schematic_definition::get_components;
use crate::port_entity::PortEntity;
use crate::schematic::OutputReady;
use crate::serialize;
use crate::vino_component;
use crate::Invocation;
use crate::Result;
use crate::RuntimeManifest;
use actix::dev::Message;
use actix::prelude::*;
use futures::future::try_join_all;
use serde::Serialize;
use vino_component::BoxedComponent;
use vino_guest::OutputPayload;

#[derive(Debug, Clone, Default)]
pub struct ActorPorts {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl ActorPorts {
    pub fn new<I, A, O, B>(inputs: I, outputs: O) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<String>,
        O: IntoIterator<Item = B>,
        B: Into<String>,
    {
        ActorPorts {
            inputs: inputs.into_iter().map(Into::into).collect(),
            outputs: outputs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ComponentMetadata {
    ports: super::ActorPorts,
}
pub struct Network {
    // host_labels: HashMap<String, String>,
    // kp: Option<KeyPair>,
    // started: std::time::Instant,
    allow_latest: bool,
    allowed_insecure: Vec<String>,
    registry: ComponentRegistry,
    host_id: String,
    seed: String,
    schematics: HashMap<String, Addr<Schematic>>,
    manifest: RuntimeManifest,
}

#[derive(Default, Clone)]
pub struct ComponentRegistry {
    pub components: HashMap<String, BoxedComponent>,
    pub receivers: HashMap<String, Recipient<Invocation>>,
}

#[derive(Debug, Clone)]
pub struct ComponentMetadata2 {
    pub ports: super::ActorPorts,
    pub addr: Recipient<Invocation>,
}

pub type MetadataMap = HashMap<String, ComponentMetadata2>;

impl ComponentRegistry {
    pub fn get_metadata(&self) -> Result<MetadataMap> {
        let mut map = MetadataMap::new();
        for (name, component) in &self.components {
            let recipient = self.receivers.get(name);
            if recipient.is_none() {
                return Err(anyhow!("Could not get recipient for {}", name).into());
            }
            let recipient = recipient.unwrap();

            map.insert(
                name.to_string(),
                ComponentMetadata2 {
                    ports: ActorPorts {
                        inputs: component.get_inputs(),
                        outputs: component.get_outputs(),
                    },
                    addr: recipient.clone(),
                },
            );
        }
        trace!("Made metadata map : {:?}", map);
        Ok(map)
    }
}

impl Default for Network {
    fn default() -> Self {
        Network {
            // host_labels: HashMap::new(),
            // kp: None,
            // started: std::time::Instant::now(),
            allow_latest: false,
            allowed_insecure: vec![],
            registry: ComponentRegistry::default(),
            host_id: "".to_string(),
            seed: "".to_string(),
            schematics: HashMap::new(),
            manifest: RuntimeManifest::default(),
        }
    }
}

impl Supervised for Network {}

impl SystemService for Network {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        trace!("Network started");
        ctx.set_mailbox_capacity(1000);
    }
}

impl HostLocalSystemService for Network {}

impl Actor for Network {
    type Context = Context<Self>;
}

impl Network {}

pub async fn request<T: AsRef<str> + Display>(
    network: &Addr<Network>,
    schematic: T,
    data: HashMap<T, impl Serialize>,
) -> Result<HashMap<String, MessagePayload>> {
    let serialized_data: HashMap<String, Vec<u8>> = data
        .iter()
        .map(|(k, v)| Ok((k.to_string(), serialize(&v)?)))
        .filter_map(Result::ok)
        .collect();

    let time = std::time::Instant::now();
    let result = network
        .send(Request {
            schematic: schematic.to_string(),
            data: serialized_data,
        })
        .await??;
    trace!(
        "result for {} took {} microseconds",
        schematic,
        time.elapsed().as_micros()
    );
    trace!("{:?}", result);
    Ok(result)
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
    pub host_id: String,
    pub seed: String,
    pub manifest: RuntimeManifest,
}

impl Handler<Initialize> for Network {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Network initializing on {}", msg.host_id);
        self.host_id = msg.host_id;
        self.seed = msg.seed;
        self.manifest = msg.manifest;
        let manifest = self.manifest.clone();
        let host_id = self.host_id.to_string();
        let seed = self.seed.clone();

        let allow_latest = self.allow_latest;
        let allowed_insecure = self.allowed_insecure.clone();
        let schematics = manifest.schematics.clone();

        Box::pin(
            async move {
                trace!("Getting components");
                get_components(&manifest, seed.clone(), allow_latest, &allowed_insecure).await
            }
            .into_actor(self)
            .then(move |components, network, _ctx| {
                match components {
                    Ok(components) => {
                        for (component, recipient) in components {
                            network
                                .registry
                                .receivers
                                .insert(component.public_key(), recipient);
                            network
                                .registry
                                .components
                                .insert(component.public_key(), component);
                        }
                    }
                    Err(e) => {
                        error!("Failed to load all components: {}", e);
                    }
                };
                let metadata = network.registry.get_metadata().unwrap_or_default();
                let inits: Vec<(Addr<Schematic>, super::schematic::Initialize)> = schematics
                    .iter()
                    .map(|schem_def| {
                        let schematic = Schematic::default().start();
                        network
                            .schematics
                            .insert(schem_def.name.to_string(), schematic.clone());
                        (
                            schematic,
                            super::schematic::Initialize {
                                host_id: host_id.to_string(),
                                schematic: schem_def.clone(),
                                components: metadata.clone(),
                                seed: network.seed.clone(),
                            },
                        )
                    })
                    .collect();
                async move {
                    match try_join_all(inits.into_iter().map(|(schem, msg)| schem.send(msg))).await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            Err(anyhow!("Error initializing schematics {}", e.to_string()).into())
                        }
                    }
                }
                .into_actor(network)
            }),
        )
    }
}

impl Handler<OutputReady> for Network {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: OutputReady, _ctx: &mut Context<Self>) -> Self::Result {
        let schematic_name = msg.port.schematic.to_string();
        let receiver = self.schematics.get(&schematic_name).cloned();
        Box::pin(
            async move {
                match receiver {
                    Some(schematic) => Ok(schematic.send(msg).await??),
                    None => Err(anyhow!("Pushing output failed").into()),
                }
            }
            .into_actor(self),
        )
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct WapcOutputReady {
    pub port: PortEntity,
    pub tx_id: String,
    pub payload: Vec<u8>,
}

impl Handler<WapcOutputReady> for Network {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: WapcOutputReady, _ctx: &mut Context<Self>) -> Self::Result {
        let schematic_name = msg.port.schematic.to_string();
        let receiver = self.schematics.get(&schematic_name).cloned();
        let payload = msg.payload;
        let tx_id = msg.tx_id;
        let port = msg.port;
        let data = deserialize::<OutputPayload>(&payload)
            .map_err(|e| MessagePayload::Error(e.to_string()));

        let message_payload = match data {
            Ok(payload) => match payload {
                OutputPayload::Bytes(b) => MessagePayload::Bytes(b),
                OutputPayload::Exception(e) => MessagePayload::Exception(e),
                OutputPayload::Error(e) => MessagePayload::Error(e),
            },
            Err(e) => e,
        };
        Box::pin(
            async move {
                match receiver {
                    Some(schematic) => Ok(schematic
                        .send(OutputReady {
                            port,
                            tx_id,
                            payload: message_payload,
                        })
                        .await??),
                    None => Err(anyhow!("Pushing output failed").into()),
                }
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, MessagePayload>>")]
pub(crate) struct Request {
    pub schematic: String,
    pub data: HashMap<String, Vec<u8>>,
}

impl Handler<Request> for Network {
    type Result = ResponseActFuture<Self, Result<HashMap<String, MessagePayload>>>;

    fn handle(&mut self, msg: Request, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Requesting schematic '{}'", msg.schematic);
        let schematic_name = msg.schematic;
        let payload = msg.data;

        let tx_id = uuid::Uuid::new_v4();
        trace!("Invoking schematic '{}'", schematic_name);
        let schematic = self.schematics.get(&schematic_name).cloned();

        let request = super::schematic::Request {
            tx_id: tx_id.to_string(),
            schematic: schematic_name.to_string(),
            payload,
        };

        Box::pin(
            async move {
                if let Some(schematic) = schematic {
                    match schematic.send(request).await? {
                        Ok(payload) => Ok(deserialize(&payload.payload)?),
                        Err(e) => Err(e.to_string().into()),
                    }
                } else {
                    Err(anyhow!("Schematic '{}' not found", schematic_name).into())
                }
            }
            .into_actor(self),
        )
    }
}

#[cfg(test)]
mod test {
    use actix::Addr;
    use maplit::hashmap;
    use test_env_log::test;

    use super::super::dispatch::MessagePayload;
    use super::*;
    use crate::hlreg::HostLocalSystemService;
    use crate::network::Initialize;
    use crate::serdes::serialize;
    use crate::RuntimeManifest;
    use wascap::prelude::KeyPair;

    async fn init_network(yaml: &str) -> Result<Addr<Network>> {
        let manifest = serde_yaml::from_str::<RuntimeManifest>(&yaml)?;
        trace!("applying manifest");
        let kp = KeyPair::new_server();

        let network = super::Network::from_hostlocal_registry(&kp.public_key());
        network
            .send(Initialize {
                host_id: kp.public_key(),
                seed: kp.seed()?,
                manifest,
            })
            .await??;
        trace!("manifest applied");
        Ok(network)
    }

    #[test_env_log::test(actix_rt::test)]
    async fn native_component() -> crate::Result<()> {
        let network = init_network(include_str!("./test/native-component.yaml")).await?;

        let data = hashmap! {
            "left" => 42,
            "right" => 302309,
        };

        let mut result = request(&network, "test", data).await?;

        trace!("result: {:?}", result);
        let output: MessagePayload = result.remove("output").unwrap();
        assert_eq!(
            output,
            MessagePayload::Bytes(serialize(42 + 302309 + 302309)?)
        );
        Ok(())
    }

    #[test_env_log::test(actix_rt::test)]
    async fn wapc_component() -> crate::Result<()> {
        let network = init_network(include_str!("./test/wapc-component.yaml")).await?;

        let data = hashmap! {
            "input" => "1234567890",
        };

        let mut result = request(&network, "test", data).await?;

        let output: MessagePayload = result.remove("output").unwrap();
        assert_eq!(output, MessagePayload::Bytes(serialize("1234567890")?));

        let data = hashmap! {
            "input" => "1234",
        };
        let mut result = request(&network, "test", data).await?;

        let output: MessagePayload = result.remove("output").unwrap();
        assert_eq!(
            output,
            MessagePayload::Exception("Password needs to be longer than 8 characters".to_string())
        );

        Ok(())
    }

    #[test_env_log::test(actix_rt::test)]
    async fn short_circuit() -> crate::Result<()> {
        let network = init_network(include_str!("./test/short-circuit.yaml")).await?;

        trace!("requesting schematic execution");
        let data = hashmap! {
            "input_port1" => "short",
        };

        let mut result = request(&network, "test", data).await?;

        trace!("result: {:?}", result);
        let output1: MessagePayload = result.remove("output1").unwrap();
        assert_eq!(
            output1,
            MessagePayload::Exception("Password needs to be longer than 8 characters".to_string())
        );
        Ok(())
    }

    #[test_env_log::test(actix_rt::test)]
    async fn multiple_schematics() -> crate::Result<()> {
        let network = init_network(include_str!("./test/multiple-schematics.yaml")).await?;

        let data = hashmap! {
            "left" => 42,
            "right" => 302309,
        };

        let mut result = request(&network, "first_schematic", data).await?;

        trace!("result: {:?}", result);
        let output: MessagePayload = result.remove("output").unwrap();
        assert_eq!(output, MessagePayload::Bytes(serialize(42 + 302309)?));

        let data = hashmap! {
            "input" => "some string",
        };

        let mut result = request(&network, "second_schematic", data).await?;

        trace!("result: {:?}", result);
        let output: MessagePayload = result.remove("output").unwrap();
        assert_eq!(output, MessagePayload::Bytes(serialize("some string")?));
        Ok(())
    }
}
