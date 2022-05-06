use tonic::Status;
use vino_rpc::rpc::Output;
use vino_transport::MessageTransport;

pub(crate) fn make_output(port: &str, inv_id: &str, payload: MessageTransport) -> Result<Output, Status> {
  Ok(Output {
    port: port.to_owned(),
    invocation_id: inv_id.to_owned(),
    payload: Some(payload.into()),
  })
}
