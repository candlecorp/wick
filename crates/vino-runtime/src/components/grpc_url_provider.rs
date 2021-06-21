// use actix::fut::ok;
// use actix::prelude::*;
// use vino_rpc::component_rpc_client::ComponentRpcClient;

// use crate::dispatch::{
//   Invocation,
//   InvocationResponse,
//   MessagePayload,
//   VinoEntity,
// };
// use crate::Result;

// #[derive(Default, Debug)]
// pub struct GrpcUrlProvider {
//   namespace: String,
//   state: Option<State>,
//   seed: String,
// }

// #[derive(Debug)]
// struct State {
//   address: String,
// }

// impl Actor for GrpcUrlProvider {
//   type Context = Context<Self>;

//   fn started(&mut self, _ctx: &mut Self::Context) {
//     trace!("GRPC Provider started");
//   }

//   fn stopped(&mut self, _ctx: &mut Self::Context) {}
// }

// #[derive(Message)]
// #[rtype(result = "Result<()>")]
// pub(crate) struct Initialize {
//   pub(crate) namespace: String,
//   pub(crate) address: String,
//   pub(crate) _client: String,
//   pub(crate) signing_seed: String,
// }

// impl Handler<Initialize> for GrpcUrlProvider {
//   type Result = ResponseActFuture<Self, Result<()>>;

//   fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
//     trace!("Native actor initialized");
//     self.namespace = msg.namespace;
//     self.seed = msg.signing_seed;

//     let addr = msg.address;

//     Box::pin(
//       ComponentRpcClient::connect(addr)
//         .into_actor(self)
//         .then(|_client, _provider, _ctx| ok(())),
//     )
//   }
// }

// impl Handler<Invocation> for GrpcUrlProvider {
//   type Result = InvocationResponse;

//   fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
//     trace!(
//       "Native actor Invocation - From {} to {}",
//       msg.origin.url(),
//       msg.target.url()
//     );
//     // let target = msg.target.url();

//     // let inv_id = msg.id;

//     if let VinoEntity::Provider(name) = &msg.target {
//       trace!("Handling provider invocation for name: {}", name);

//       if let MessagePayload::MultiBytes(_payload) = msg.msg {
//         InvocationResponse::error(msg.tx_id, "todo".to_string())
//       } else {
//         InvocationResponse::error(
//           msg.tx_id,
//           "Invalid payload sent from native actor".to_string(),
//         )
//       }
//     } else {
//       InvocationResponse::error(
//         msg.tx_id,
//         "Sent invocation for incorrect entity".to_string(),
//       )
//     }
//   }
// }

// #[cfg(test)]
// mod test {

//   use std::net::{
//     IpAddr,
//     Ipv4Addr,
//     SocketAddr,
//   };
//   use std::str::FromStr;
//   use std::sync::Arc;

//   use super::*;

//   async fn make_grpc_server() {
//     let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str("127.0.0.1")), 54321);

//     trace!("Binding to {:?}", addr.to_string());

//     let component_service = ComponentService { provider };

//     let svc = ComponentRpcServer::new(component_service);

//     Server::builder().add_service(svc).serve(addr).await?;

//     trace!("Server started");
//   }

//   #[test_env_log::test(actix_rt::test)]
//   async fn test_init() -> Result<()> {
//     let init_handle = init(
//       Arc::new(Mutex::new(Provider::default())),
//       Some(Options {
//         port: 12345,
//         address: Ipv4Addr::from_str("127.0.0.1")?,
//       }),
//     );
//     tokio::select! {
//         _ = tokio::time::sleep(Duration::from_secs(1)) => {
//             println!("timeout reached");
//         }
//         _ = init_handle => {
//             panic!("server died");
//         }
//     };

//     trace!("test_init");
//     let provider = GrpcUrlProvider::default();
//     let addr = provider.start();
//     let mut schematic_def = Initialize {
//       namespace: "test",
//       address: "",
//       _client: (),
//       signing_seed: (),
//     };

//     let hostkey = KeyPair::new_server();

//     addr
//       .send(Initialize {
//         network: Network::from_hostlocal_registry(&KeyPair::new_server().public_key()),
//         host_id: "test".to_string(),
//         schematic: schematic_def,
//         seed: hostkey.seed()?,
//         allow_latest: true,
//         allowed_insecure: vec![],
//       })
//       .await??;
//     let mut input: HashMap<String, Vec<u8>> = HashMap::new();
//     input.insert("input".to_string(), vec![20]);
//     let response = addr
//       .send(super::Request {
//         tx_id: "some_id".to_string(),
//         schematic: "logger".to_string(),
//         payload: input,
//       })
//       .await?;
//     println!("{:?}", response);

//     Ok(())
//   }
// }
