use rmp_futures::rpc::decode::RpcParamsFuture;
use rmp_futures::rpc::MsgId;
use tokio::io::AsyncRead;
use vino_runtime::deserialize;

use crate::rpc::VinoRpcMessage;
use crate::{Error, Result};

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

#[cfg(test)]
mod tests {

    // #[test_env_log::test(tokio::test)]
    // async fn testthing() -> anyhow::Result<()> {
    //     Ok(())
    // }
}
