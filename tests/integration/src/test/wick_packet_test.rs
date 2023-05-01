use anyhow::Result;
use wick_packet::{packet_stream, StreamMap};

#[test_logger::test(tokio::test)]
async fn test_streammap() -> Result<()> {
  let stream = packet_stream!(("a", 1), ("b", 2), ("c", 3));
  let mut map = StreamMap::from_stream(stream, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
  let mut set = map.next_set().await.unwrap().unwrap();
  assert_eq!(set.len(), 3);
  assert_eq!(set.remove("a").unwrap().payload.deserialize::<i32>().unwrap(), 1);
  assert_eq!(set.remove("b").unwrap().payload.deserialize::<i32>().unwrap(), 2);
  assert_eq!(set.remove("c").unwrap().payload.deserialize::<i32>().unwrap(), 3);

  Ok(())
}
