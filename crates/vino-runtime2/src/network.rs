use std::collections::HashMap;

use super::schematic::Schematic;
use crate::deserialize;
use crate::dispatch::MessagePayload;
use crate::hlreg::HostLocalSystemService;
use crate::manifest_definition::HostManifest;
use crate::schematic::OutputReady;
use crate::schematic_definition::get_components;
use crate::vino_component;
use crate::Invocation;
use crate::Result;
use actix::dev::Message;
use actix::prelude::*;
use futures::future::try_join_all;
use vino_component::BoxedComponent;

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
    manifest: HostManifest,
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
            manifest: HostManifest::default(),
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

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
    pub host_id: String,
    pub seed: String,
    pub manifest: HostManifest,
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

        let allow_latest = self.allow_latest;
        let allowed_insecure = self.allowed_insecure.clone();
        let schematics = manifest.schematics.clone();

        Box::pin(
            async move {
                trace!("Getting components");
                get_components(&manifest, allow_latest, &allowed_insecure).await
            }
            .into_actor(self)
            .then(move |components, network, _ctx| {
                if let Ok(components) = components {
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
            schematic: schematic_name,
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
                    Err("Foo".to_string().into())
                }
            }
            .into_actor(self),
        )
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, time::Instant};
    use test_env_log::test;

    use wascap::prelude::KeyPair;

    use super::super::dispatch::MessagePayload;
    use crate::hlreg::HostLocalSystemService;
    use crate::network::Initialize;
    use crate::serdes::serialize;
    use crate::{deserialize, HostManifest};

    #[test_env_log::test(actix_rt::test)]
    async fn network_init() -> crate::Result<()> {
        let yaml = "
---
schematics:
  - name: test
    components:
      add:
        ref: vino::add
    connections:
      - from:
          instance: vino::schematic_input
          port: left
        to:
          instance: add
          port: left
      - from:
          instance: vino::schematic_input
          port: right
        to:
          instance: add
          port: right
      - from:
          instance: add
          port: output
        to:
          instance: vino::schematic_output
          port: output
";
        let manifest = serde_yaml::from_str::<HostManifest>(&yaml)?;
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

        trace!("requesting schematic execution");

        let time = Instant::now();
        let data: HashMap<String, Vec<u8>> = vec![
            ("left".to_string(), serialize(42)?),
            ("right".to_string(), serialize(302309)?),
        ]
        .iter()
        .cloned()
        .collect();

        let result = network
            .send(super::Request {
                schematic: "test".to_string(),
                data,
            })
            .await??;
        trace!(
            "result: {:?} took {} microseconds",
            result,
            time.elapsed().as_micros()
        );
        trace!("result: {:?}", result);
        let output: &MessagePayload = result.get("output").unwrap();
        if let MessagePayload::Bytes(data) = output {
            let result: u64 = deserialize(data)?;
            assert_eq!(result, 42 + 302309);
        } else {
            panic!();
        }
        Ok(())
    }
}
