#[macro_export]
macro_rules! request_response {
  ($name:ident, $handler:ident=> {
    inputs: {$($ikey:ident => $ity:ty),* $(,)?},
    output: $okey:expr,
  }) => {
    pub(crate) async fn $name(mut stream: PacketStream) -> Result<PacketStream, Box<dyn std::error::Error + Send + Sync>> {
      #[allow(unused_parens)]
      let ($(mut $ikey),*) = fan_out!(stream, $(stringify!($ikey)),*);
      let (tx, rx) = PacketStream::new_channels();
      tokio::spawn(async move {
      #[allow(unused_parens)]
        while let ($(Some(Ok($ikey))),*) = ($($ikey.next().await),*) {
          $(let $ikey = $ikey.deserialize::<$ity>()?;)*
          let output = $handler($($ikey,)*).await?;
          tx.send(Packet::encode($okey, output))?;
        }
        tx.send(Packet::done($okey))?;
        Ok::<_, wasmflow_packet_stream::Error>(())
      });
      Ok(rx.into())
    }
  };
}
