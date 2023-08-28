// use std::sync::Arc;

// use anyhow::Result;
// use futures::{Stream, StreamExt};
// use tokio_stream::wrappers::ReceiverStream;
// use tonic::{IntoStreamingRequest, Status};
// use wick_invocation_server::InvocationServer;
// use wick_packet::{packets, Entity, Invocation, Packet};
// use wick_rpc::rpc::invocation_service_server::InvocationService;
// use wick_rpc::rpc::{InvocationRequest, StatsResponse};

// use super::NativeComponent;

// fn get_server() -> InvocationServer {
//   let collection = Arc::new(NativeComponent::default());
//   InvocationServer::new(collection)
// }

// // fn rpc_packets() -> impl tonic::IntoStreamingRequest<Message = InvocationRequest> {
// fn rpc_packets() -> impl Stream<Item = InvocationRequest> {
//   let invocation = Invocation::new(Entity::test("stats"), Entity::local("test-component"), None);
//   let packet = Packet::encode("input", "hello");
//   let messages = vec![
//     wick_rpc::rpc::InvocationRequest {
//       data: Some(wick_rpc::rpc::invocation_request::Data::Invocation(invocation.into())),
//     },
//     wick_rpc::rpc::InvocationRequest {
//       data: Some(wick_rpc::rpc::invocation_request::Data::Packet(packet.into())),
//     },
//   ];

//   tokio_stream::iter(messages)
// }

// async fn make_test_invocation(
//   server: &InvocationServer,
// ) -> Result<tonic::Response<ReceiverStream<Result<wick_rpc::rpc::Packet, Status>>>> {
//   let payload = packets!(("input", "hello"));

//   let s = tonic::Request::new(rpc_packets());
//   let method = InvokeSvc(inner);
//   let codec = tonic::codec::ProstCodec::default();
//   let mut grpc = tonic::server::Grpc::new(codec);
//   let res = grpc.streaming(method, s).await;

//   // let s = rpc_packets().into_streaming_request();
//   // let s: tonic::Request<tonic::Streaming<InvocationRequest>> = rpc_packets().into_streaming_request();
//   // let s: tonic::Streaming<InvocationRequest> = rpc_packets().into_streaming_request();
//   let result = server.invoke(s.into_streaming_request()).await?;
//   Ok(result)
// }

// async fn get_test_stats() -> Result<StatsResponse> {
//   let server = get_server();
//   let _response = make_test_invocation(&server).await?;
//   let _response = make_test_invocation(&server).await?;
//   let _response = make_test_invocation(&server).await?;

//   let stats_request = tonic::Request::new(wick_rpc::rpc::StatsRequest {});

//   let stats = server.stats(stats_request).await?;
//   Ok(stats.into_inner())
// }

// #[test_logger::test(tokio::test)]
// async fn test_stats() -> Result<()> {
//   let mut stats = get_test_stats().await?;

//   let stat = stats.stats[0].execution_statistics.take().unwrap();

//   //three runs must be longer than two runs
//   assert!(stat.total > stat.min + stat.max);

//   Ok(())
// }
