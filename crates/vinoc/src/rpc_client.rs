use std::net::Ipv4Addr;

use tonic::transport::Channel;
use vino_rpc::rpc::invocation_service_client::InvocationServiceClient;

use crate::Result;

pub(crate) async fn rpc_client(
  address: Ipv4Addr,
  port: u16,
) -> Result<InvocationServiceClient<Channel>> {
  Ok(InvocationServiceClient::connect(format!("https://{}:{}", address, port)).await?)
}
