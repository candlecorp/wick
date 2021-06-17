use rmp_futures::rpc::decode::RpcParamsFuture;
use rmp_futures::rpc::MsgId;
use tokio::io::AsyncRead;
use vino_runtime::deserialize;

use crate::rpc::VinoRpcMessage;
use crate::{
  rpc,
  Error,
  Result,
};

pub async fn handle<R>(
  method: &str,
  id: MsgId,
  params: RpcParamsFuture<R>,
) -> Result<(Option<VinoRpcMessage>, R)>
where
  R: AsyncRead + Unpin + std::fmt::Debug + Send,
{
  match method {
    rpc::OP_INVOKE => handler(id, params).await,
    rpc::OP_ERROR => handler(id, params).await,
    rpc::OP_CLOSE => handler(id, params).await,
    rpc::OP_OUTPUT => handler(id, params).await,
    rpc::OP_PING => {
      trace!("<PING>");
      let (message, reader) = handle_string(id, params).await?;
      Ok((Some(VinoRpcMessage::Ping(message)), reader))
    }
    rpc::OP_PONG => {
      trace!("<PONG>");
      let (message, reader) = handle_string(id, params).await?;
      Ok((Some(VinoRpcMessage::Pong(message)), reader))
    }
    rpc::OP_SHUTDOWN => Ok((
      Some(VinoRpcMessage::Shutdown),
      params.params().await?.skip().await?,
    )),
    _ => panic!("unhandled method {}", method),
  }
}

pub async fn handler<R>(
  id: MsgId,
  params: RpcParamsFuture<R>,
) -> Result<(Option<VinoRpcMessage>, R)>
where
  R: AsyncRead + Unpin + std::fmt::Debug,
{
  let params = params.params().await?;
  let (msg, reader) = match params.last() {
    rmp_futures::MsgPackOption::Some(params) => {
      let first = params.decode().await?;
      let string_fut = first
        .into_string()
        .ok_or(Error::RpcMessageError("Failed to deserialize"))?;
      let mut buf = vec![0; string_fut.len()];
      let reader = string_fut.read_all(&mut buf).await?;
      let msg: VinoRpcMessage = deserialize(&buf)?;
      (Some(msg), reader)
    }
    rmp_futures::MsgPackOption::End(reader) => (None, reader),
  };
  trace!("Msg id {} param = {:?}", id, msg);
  Ok((msg, reader))
}

pub async fn handle_string<R>(id: MsgId, params: RpcParamsFuture<R>) -> Result<(String, R)>
where
  R: AsyncRead + Unpin + std::fmt::Debug,
{
  let params = params.params().await?;
  let (msg, reader) = match params.last() {
    rmp_futures::MsgPackOption::Some(params) => {
      let first = params.decode().await?;
      let string_fut = first
        .into_string()
        .ok_or(Error::RpcMessageError("Failed to deserialize"))?;
      let (msg, reader) = string_fut.into_string().await?;
      (msg, reader)
    }
    rmp_futures::MsgPackOption::End(reader) => ("Unreachable".to_string(), reader),
  };
  trace!("Msg id {} param = {:?}", id, msg);
  Ok((msg, reader))
}

#[cfg(test)]
mod tests {

  // #[test_env_log::test(tokio::test)]
  // async fn testthing() -> anyhow::Result<()> {
  //     Ok(())
  // }
}
