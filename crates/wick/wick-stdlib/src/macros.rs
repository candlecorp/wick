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
        let error = loop {
          #[allow(unused_parens)]
          let ($($ikey),*) = ($($ikey.next().await),*);
          #[allow(unused_parens)]
          if let ($(Some(Ok($ikey))),*) = ($($ikey),*) {
            $(let $ikey = match $ikey.deserialize::<$ity>(){Ok(v)=>v,Err(e)=>break Some(e)};)*
            let output = match $handler($($ikey,)*) { Ok(o)=>o, Err(e)=> break(Some(e))};
            tx.send(Packet::encode($okey, output))?;
          } else {
            break None;
          }
        };
        if let Some(error) = error {
          tx.send(Packet::err($okey, error.to_string()))?;
        }
        tx.send(Packet::done($okey))?;
        Ok::<_, wick_packet::Error>(())
      });
      Ok(rx.into())
    }
  };
}
