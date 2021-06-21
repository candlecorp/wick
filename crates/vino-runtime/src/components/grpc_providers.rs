// use std::env::temp_dir;
// use std::path::PathBuf;

// use actix::prelude::*;
// use wascap::prelude::{
//   Claims,
//   KeyPair,
// };

// use crate::dispatch::{
//   Invocation,
//   InvocationResponse,
// };
// use crate::{
//   Result,
//   VinoEntity,
// };

// #[derive(Clone, Debug)]
// pub struct ExternalProvider {
//   pub(crate) link_name: String,
//   pub(crate) claims: Claims<wascap::jwt::CapabilityProvider>,
//   pub(crate) bytes: Option<Vec<u8>>,
// }

// impl ExternalProvider {
//   /// Returns the path where this capability provider's binary resides/should reside for cache purposes.
//   pub fn cache_path(&self) -> PathBuf {
//     let mut path = temp_dir();
//     path.push("vino");
//     path.push(&self.claims.subject);
//     path.push(format!(
//       "{}",
//       self.claims.metadata.as_ref().unwrap().rev.unwrap_or(0)
//     ));
//     path.push(Self::native_target());
//     path
//   }
//   pub fn native_target() -> String {
//     format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
//   }
// }

// #[derive(Message, Debug)]
// #[rtype(result = "Result<VinoEntity>")]
// pub(crate) struct Initialize {
//   pub(crate) seed: String,
//   pub(crate) image_ref: Option<String>,
// }

// // trait ExternalProvider {}

// struct State {
//   #[allow(dead_code)]
//   kp: KeyPair,
//   #[allow(dead_code)]
//   plugin: Box<ExternalProvider>,
//   #[allow(dead_code)]
//   image_ref: Option<String>,
// }

// pub(crate) struct GrpcProvider {
//   state: Option<State>,
// }

// impl GrpcProvider {}

// impl Actor for GrpcProvider {
//   type Context = SyncContext<Self>;

//   fn started(&mut self, _ctx: &mut Self::Context) {
//     info!("Native provider host started");
//   }

//   fn stopped(&mut self, _ctx: &mut Self::Context) {
//     if self.state.is_none() {
//       return;
//     }
//     let _state = self.state.as_mut().unwrap();

//     // state.plugin.stop(); // Tell the provider to clean up, dispose of resources, stop threads, etc
//   }
// }

// impl Handler<Initialize> for GrpcProvider {
//   type Result = Result<VinoEntity>;

//   fn handle(&mut self, _msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
//     // let (library, plugin) = match extrude(&msg.cap) {
//     //   Ok((l, r)) => (l, r),
//     //   Err(e) => {
//     //     error!("Failed to extract plugin from provider: {}", e);
//     //     ctx.stop();
//     //     return Err("Failed to extract plugin from provider".into());
//     //   }
//     // };

//     Ok(VinoEntity::default())
//   }
// }

// impl Handler<Invocation> for GrpcProvider {
//   type Result = InvocationResponse;

//   fn handle(&mut self, _inv: Invocation, _ctx: &mut Self::Context) -> Self::Result {
//     // let state = self.state.as_ref().unwrap();
//     InvocationResponse::success("TODO".to_string(), vec![])
//   }
// }

// #[allow(dead_code)]
// pub(crate) fn write_provider_to_disk(provider: &ExternalProvider) -> Result<()> {
//   if let Some(ref bytes) = provider.bytes {
//     use std::io::Write;
//     let path = provider.cache_path();
//     let mut parent_dir = path.clone();
//     parent_dir.pop();
//     std::fs::create_dir_all(parent_dir)?;
//     debug!("Caching provider to {}", path.display());
//     let mut file = std::fs::File::create(&path)?;
//     Ok(file.write_all(bytes)?)
//   } else {
//     Err("No bytes to cache".into())
//   }
// }

// #[allow(dead_code)]
// fn extrude(provider: &ExternalProvider) -> Result<()> {
//   if provider.bytes.is_some() {
//     let path = provider.cache_path();
//     // If this file is already on disk, don't overwrite
//     if path.exists() {
//       debug!("Using cache at {}", path.display());
//     } else {
//       write_provider_to_disk(provider)?;
//     }
//     // #[cfg(target_os = "linux")]
//     // #[cfg(not(target_os = "linux"))]
//     // Ok((Some(library), plugin))
//   }
//   Ok(())
// }

// #[cfg(test)]
// mod test {
//   // use actix::prelude::*;

//   #[actix_rt::test]
//   async fn test_extras_actor() {}
// }
