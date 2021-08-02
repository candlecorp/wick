#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageKind {
  #[prost(enumeration = "message_kind::Kind", tag = "1")]
  pub kind: i32,
  #[prost(oneof = "message_kind::Data", tags = "2, 3, 4, 5")]
  pub data: ::core::option::Option<message_kind::Data>,
}
/// Nested message and enum types in `MessageKind`.
pub mod message_kind {
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
  #[repr(i32)]
  pub enum Kind {
    Invalid = 0,
    Error = 1,
    Exception = 2,
    Test = 3,
    MessagePack = 4,
    Signal = 5,
    Json = 6,
  }
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
  #[repr(i32)]
  pub enum OutputSignal {
    Done = 0,
    OpenBracket = 1,
    CloseBracket = 2,
  }
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Data {
    #[prost(bytes, tag = "2")]
    Messagepack(::prost::alloc::vec::Vec<u8>),
    #[prost(string, tag = "3")]
    Message(::prost::alloc::string::String),
    #[prost(enumeration = "OutputSignal", tag = "4")]
    Signal(i32),
    #[prost(string, tag = "5")]
    Json(::prost::alloc::string::String),
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Invocation {
  #[prost(string, tag = "1")]
  pub origin: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub target: ::prost::alloc::string::String,
  #[prost(map = "string, message", tag = "3")]
  pub msg: ::std::collections::HashMap<::prost::alloc::string::String, MessageKind>,
  #[prost(string, tag = "4")]
  pub id: ::prost::alloc::string::String,
  #[prost(string, tag = "7")]
  pub network_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Output {
  #[prost(string, tag = "1")]
  pub port: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub invocation_id: ::prost::alloc::string::String,
  #[prost(message, optional, tag = "3")]
  pub payload: ::core::option::Option<MessageKind>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatsRequest {
  #[prost(oneof = "stats_request::Kind", tags = "1, 2")]
  pub kind: ::core::option::Option<stats_request::Kind>,
}
/// Nested message and enum types in `StatsRequest`.
pub mod stats_request {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Format {}
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Component {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub format: ::core::option::Option<Format>,
  }
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Kind {
    #[prost(message, tag = "1")]
    All(Format),
    #[prost(message, tag = "2")]
    Component(Component),
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResponse {
  #[prost(message, repeated, tag = "1")]
  pub components: ::prost::alloc::vec::Vec<Component>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Component {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(enumeration = "component::ComponentKind", tag = "2")]
  pub kind: i32,
  #[prost(message, repeated, tag = "3")]
  pub inputs: ::prost::alloc::vec::Vec<component::Port>,
  #[prost(message, repeated, tag = "4")]
  pub outputs: ::prost::alloc::vec::Vec<component::Port>,
  #[prost(message, repeated, tag = "5")]
  pub providers: ::prost::alloc::vec::Vec<Provider>,
}
/// Nested message and enum types in `Component`.
pub mod component {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Port {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub r#type: ::prost::alloc::string::String,
  }
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
  #[repr(i32)]
  pub enum ComponentKind {
    Component = 0,
    Schematic = 1,
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Provider {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(message, repeated, tag = "2")]
  pub components: ::prost::alloc::vec::Vec<Component>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatsResponse {
  #[prost(message, repeated, tag = "1")]
  pub stats: ::prost::alloc::vec::Vec<Statistic>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Statistic {
  #[prost(uint64, tag = "1")]
  pub num_calls: u64,
}
#[doc = r" Generated client implementations."]
pub mod invocation_service_client {
  #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
  use tonic::codegen::*;
  #[doc = " Interface exported by the server."]
  #[derive(Debug, Clone)]
  pub struct InvocationServiceClient<T> {
    inner: tonic::client::Grpc<T>,
  }
  impl InvocationServiceClient<tonic::transport::Channel> {
    #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
      D: std::convert::TryInto<tonic::transport::Endpoint>,
      D::Error: Into<StdError>,
    {
      let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
      Ok(Self::new(conn))
    }
  }
  impl<T> InvocationServiceClient<T>
  where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + Send + Sync + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
  {
    pub fn new(inner: T) -> Self {
      let inner = tonic::client::Grpc::new(inner);
      Self { inner }
    }
    pub fn with_interceptor<F>(
      inner: T,
      interceptor: F,
    ) -> InvocationServiceClient<InterceptedService<T, F>>
    where
      F: FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>,
      T: tonic::codegen::Service<
        http::Request<tonic::body::BoxBody>,
        Response = http::Response<
          <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
        >,
      >,
      <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
        Into<StdError> + Send + Sync,
    {
      InvocationServiceClient::new(InterceptedService::new(inner, interceptor))
    }
    #[doc = r" Compress requests with `gzip`."]
    #[doc = r""]
    #[doc = r" This requires the server to support it otherwise it might respond with an"]
    #[doc = r" error."]
    pub fn send_gzip(mut self) -> Self {
      self.inner = self.inner.send_gzip();
      self
    }
    #[doc = r" Enable decompressing responses with `gzip`."]
    pub fn accept_gzip(mut self) -> Self {
      self.inner = self.inner.accept_gzip();
      self
    }
    pub async fn invoke(
      &mut self,
      request: impl tonic::IntoRequest<super::Invocation>,
    ) -> Result<tonic::Response<tonic::codec::Streaming<super::Output>>, tonic::Status> {
      self.inner.ready().await.map_err(|e| {
        tonic::Status::new(
          tonic::Code::Unknown,
          format!("Service was not ready: {}", e.into()),
        )
      })?;
      let codec = tonic::codec::ProstCodec::default();
      let path = http::uri::PathAndQuery::from_static("/vino.InvocationService/Invoke");
      self
        .inner
        .server_streaming(request.into_request(), path, codec)
        .await
    }
    pub async fn list(
      &mut self,
      request: impl tonic::IntoRequest<super::ListRequest>,
    ) -> Result<tonic::Response<super::ListResponse>, tonic::Status> {
      self.inner.ready().await.map_err(|e| {
        tonic::Status::new(
          tonic::Code::Unknown,
          format!("Service was not ready: {}", e.into()),
        )
      })?;
      let codec = tonic::codec::ProstCodec::default();
      let path = http::uri::PathAndQuery::from_static("/vino.InvocationService/List");
      self.inner.unary(request.into_request(), path, codec).await
    }
    pub async fn stats(
      &mut self,
      request: impl tonic::IntoRequest<super::StatsRequest>,
    ) -> Result<tonic::Response<super::StatsResponse>, tonic::Status> {
      self.inner.ready().await.map_err(|e| {
        tonic::Status::new(
          tonic::Code::Unknown,
          format!("Service was not ready: {}", e.into()),
        )
      })?;
      let codec = tonic::codec::ProstCodec::default();
      let path = http::uri::PathAndQuery::from_static("/vino.InvocationService/Stats");
      self.inner.unary(request.into_request(), path, codec).await
    }
  }
}
#[doc = r" Generated server implementations."]
pub mod invocation_service_server {
  #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
  use tonic::codegen::*;
  #[doc = "Generated trait containing gRPC methods that should be implemented for use with InvocationServiceServer."]
  #[async_trait]
  pub trait InvocationService: Send + Sync + 'static {
    #[doc = "Server streaming response type for the Invoke method."]
    type InvokeStream: futures_core::Stream<Item = Result<super::Output, tonic::Status>>
      + Send
      + Sync
      + 'static;
    async fn invoke(
      &self,
      request: tonic::Request<super::Invocation>,
    ) -> Result<tonic::Response<Self::InvokeStream>, tonic::Status>;
    async fn list(
      &self,
      request: tonic::Request<super::ListRequest>,
    ) -> Result<tonic::Response<super::ListResponse>, tonic::Status>;
    async fn stats(
      &self,
      request: tonic::Request<super::StatsRequest>,
    ) -> Result<tonic::Response<super::StatsResponse>, tonic::Status>;
  }
  #[doc = " Interface exported by the server."]
  #[derive(Debug)]
  pub struct InvocationServiceServer<T: InvocationService> {
    inner: _Inner<T>,
    accept_compression_encodings: (),
    send_compression_encodings: (),
  }
  struct _Inner<T>(Arc<T>);
  impl<T: InvocationService> InvocationServiceServer<T> {
    pub fn new(inner: T) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner);
      Self {
        inner,
        accept_compression_encodings: Default::default(),
        send_compression_encodings: Default::default(),
      }
    }
    pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
    where
      F: FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>,
    {
      InterceptedService::new(Self::new(inner), interceptor)
    }
  }
  impl<T, B> tonic::codegen::Service<http::Request<B>> for InvocationServiceServer<T>
  where
    T: InvocationService,
    B: Body + Send + Sync + 'static,
    B::Error: Into<StdError> + Send + 'static,
  {
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = Never;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: http::Request<B>) -> Self::Future {
      let inner = self.inner.clone();
      match req.uri().path() {
        "/vino.InvocationService/Invoke" => {
          #[allow(non_camel_case_types)]
          struct InvokeSvc<T: InvocationService>(pub Arc<T>);
          impl<T: InvocationService> tonic::server::ServerStreamingService<super::Invocation>
            for InvokeSvc<T>
          {
            type Response = super::Output;
            type ResponseStream = T::InvokeStream;
            type Future = BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::Invocation>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).invoke(request).await };
              Box::pin(fut)
            }
          }
          let accept_compression_encodings = self.accept_compression_encodings;
          let send_compression_encodings = self.send_compression_encodings;
          let inner = self.inner.clone();
          let fut = async move {
            let inner = inner.0;
            let method = InvokeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = tonic::server::Grpc::new(codec)
              .apply_compression_config(accept_compression_encodings, send_compression_encodings);
            let res = grpc.server_streaming(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/vino.InvocationService/List" => {
          #[allow(non_camel_case_types)]
          struct ListSvc<T: InvocationService>(pub Arc<T>);
          impl<T: InvocationService> tonic::server::UnaryService<super::ListRequest> for ListSvc<T> {
            type Response = super::ListResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::ListRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).list(request).await };
              Box::pin(fut)
            }
          }
          let accept_compression_encodings = self.accept_compression_encodings;
          let send_compression_encodings = self.send_compression_encodings;
          let inner = self.inner.clone();
          let fut = async move {
            let inner = inner.0;
            let method = ListSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = tonic::server::Grpc::new(codec)
              .apply_compression_config(accept_compression_encodings, send_compression_encodings);
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/vino.InvocationService/Stats" => {
          #[allow(non_camel_case_types)]
          struct StatsSvc<T: InvocationService>(pub Arc<T>);
          impl<T: InvocationService> tonic::server::UnaryService<super::StatsRequest> for StatsSvc<T> {
            type Response = super::StatsResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::StatsRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).stats(request).await };
              Box::pin(fut)
            }
          }
          let accept_compression_encodings = self.accept_compression_encodings;
          let send_compression_encodings = self.send_compression_encodings;
          let inner = self.inner.clone();
          let fut = async move {
            let inner = inner.0;
            let method = StatsSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = tonic::server::Grpc::new(codec)
              .apply_compression_config(accept_compression_encodings, send_compression_encodings);
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        _ => Box::pin(async move {
          Ok(
            http::Response::builder()
              .status(200)
              .header("grpc-status", "12")
              .header("content-type", "application/grpc")
              .body(empty_body())
              .unwrap(),
          )
        }),
      }
    }
  }
  impl<T: InvocationService> Clone for InvocationServiceServer<T> {
    fn clone(&self) -> Self {
      let inner = self.inner.clone();
      Self {
        inner,
        accept_compression_encodings: self.accept_compression_encodings,
        send_compression_encodings: self.send_compression_encodings,
      }
    }
  }
  impl<T: InvocationService> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(self.0.clone())
    }
  }
  impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self.0)
    }
  }
  impl<T: InvocationService> tonic::transport::NamedService for InvocationServiceServer<T> {
    const NAME: &'static str = "vino.InvocationService";
  }
}
