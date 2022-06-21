use tonic::Status;
use wasmflow_rpc::rpc::Output;
use wasmflow_sdk::v1::transport::MessageTransport;

pub(crate) fn make_output(port: &str, inv_id: &str, payload: MessageTransport) -> Result<Output, Status> {
  Ok(Output {
    port: port.to_owned(),
    invocation_id: inv_id.to_owned(),
    payload: Some(payload.into()),
  })
}
