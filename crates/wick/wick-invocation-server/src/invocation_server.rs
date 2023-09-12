use std::collections::HashMap;
use std::time::{Duration, Instant};

use flow_component::{panic_callback, SharedComponent};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Response, Status};
use wick_packet::PacketStream;
use wick_rpc::rpc::invocation_service_server::InvocationService;
use wick_rpc::rpc::{InvocationRequest, ListResponse, Packet, StatsResponse};
use wick_rpc::{rpc, DurationStatistics, Statistics};

/// A GRPC server for implementers of [flow_component::Component].
pub struct InvocationServer {
  /// The component that will handle incoming requests.
  pub collection: SharedComponent,

  stats: RwLock<HashMap<String, Statistics>>,
}

impl std::fmt::Debug for InvocationServer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InvocationServer").finish()
  }
}

impl InvocationServer {
  /// Constructor.
  #[must_use]
  pub fn new(collection: SharedComponent) -> Self {
    Self {
      collection,
      stats: RwLock::new(HashMap::new()),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum JobResult {
  Success,
  Error,
}

impl InvocationServer {
  fn record_execution<T: Into<String>>(&self, job: T, status: JobResult, time: Duration) {
    let mut stats = self.stats.write();
    let job = job.into();
    let stat = stats.entry(job.clone()).or_insert_with(Statistics::default);
    stat.runs += 1;
    if status == JobResult::Error {
      stat.errors += 1;
    }
    let durations = if stat.execution_duration.is_some() {
      let mut durations = stat.execution_duration.take().unwrap();
      if time < durations.min_time {
        durations.min_time = time;
      } else if time > durations.max_time {
        durations.max_time = time;
      }
      let average = ((durations.average_time * (stat.runs - 1)) + time) / stat.runs;
      durations.average_time = average;
      let total = durations.total_time + time;
      durations.total_time = total;

      durations
    } else {
      DurationStatistics::new(time, time, time, time)
    };
    stat.execution_duration.replace(durations);
  }
}

fn convert_invocation_stream(mut streaming: tonic::Streaming<InvocationRequest>) -> PacketStream {
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
  tokio::spawn(async move {
    while let Some(p) = streaming.next().await {
      let result = p.map_err(|e| wick_packet::Error::Component(e.to_string())).map(|p| {
        p.data.map_or_else(
          || unreachable!(),
          |p| match p {
            rpc::invocation_request::Data::Invocation(_) => unreachable!(),
            rpc::invocation_request::Data::Packet(p) => wick_packet::Packet::from(p),
          },
        )
      });

      let _ = tx.send(result);
    }
  });

  wick_packet::PacketStream::new(Box::new(tokio_stream::wrappers::UnboundedReceiverStream::new(rx)))
}

#[async_trait::async_trait]
impl InvocationService for InvocationServer {
  type InvokeStream = ReceiverStream<Result<Packet, Status>>;

  async fn invoke(
    &self,
    request: tonic::Request<tonic::Streaming<InvocationRequest>>,
  ) -> Result<Response<Self::InvokeStream>, Status> {
    let start = Instant::now();

    let (tx, rx) = mpsc::channel(4);
    let mut stream = request.into_inner();
    let first = stream.next().await;
    let invocation: wick_packet::InvocationData = if let Some(Ok(inv)) = first {
      if let Some(rpc::invocation_request::Data::Invocation(inv)) = inv.data {
        inv
          .try_into()
          .map_err(|_| Status::invalid_argument("First message must be a valid invocation"))?
      } else {
        return Err(Status::invalid_argument("First message must be an invocation"));
      }
    } else {
      return Err(Status::invalid_argument("First message must be an invocation"));
    };
    let stream = convert_invocation_stream(stream);
    let packet_stream = PacketStream::new(Box::new(stream));
    let invocation = invocation.with_stream(packet_stream);

    let op_id = invocation.target().operation_id().to_owned();

    let result = self
      .collection
      .handle(invocation, Default::default(), panic_callback())
      .await;
    if let Err(e) = result {
      let message = e.to_string();
      error!("invocation failed: {}", message);
      tx.send(Err(Status::internal(message))).await.unwrap();
      self.record_execution(op_id, JobResult::Error, start.elapsed());
    } else {
      tokio::spawn(async move {
        let mut receiver = result.unwrap();
        while let Some(next) = receiver.next().await {
          if next.is_err() {
            todo!("Handle error");
          }
          let next = next.unwrap();

          tx.send(Ok(next.into())).await.unwrap();
        }
      });
      self.record_execution(op_id, JobResult::Success, start.elapsed());
    }

    Ok(Response::new(ReceiverStream::new(rx)))
  }

  async fn list(&self, _request: tonic::Request<rpc::ListRequest>) -> Result<Response<ListResponse>, Status> {
    let response = ListResponse {
      components: vec![self.collection.signature().clone().try_into().unwrap()],
    };
    Ok(Response::new(response))
  }

  async fn stats(&self, _request: tonic::Request<rpc::StatsRequest>) -> Result<Response<StatsResponse>, Status> {
    Ok(Response::new(StatsResponse {
      stats: self.stats.read().values().cloned().map(From::from).collect(),
    }))
  }
}

#[cfg(test)]
mod tests {
  // tested in the workspace root with a native component
}
