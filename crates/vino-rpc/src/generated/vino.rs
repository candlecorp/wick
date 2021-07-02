#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputKind {
  #[prost(oneof = "output_kind::Data", tags = "1, 2, 3, 4, 5, 6")]
  pub data: ::core::option::Option<output_kind::Data>,
}
/// Nested message and enum types in `OutputKind`.
pub mod output_kind {
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Data {
    #[prost(bytes, tag = "1")]
    Messagepack(::prost::alloc::vec::Vec<u8>),
    #[prost(string, tag = "2")]
    Error(::prost::alloc::string::String),
    #[prost(string, tag = "3")]
    Exception(::prost::alloc::string::String),
    #[prost(string, tag = "4")]
    Test(::prost::alloc::string::String),
    #[prost(bool, tag = "5")]
    Invalid(bool),
    #[prost(enumeration = "super::OutputSignal", tag = "6")]
    Signal(i32),
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Invocation {
  #[prost(string, tag = "1")]
  pub origin: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub target: ::prost::alloc::string::String,
  #[prost(map = "string, bytes", tag = "3")]
  pub msg:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::vec::Vec<u8>>,
  #[prost(string, tag = "4")]
  pub id: ::prost::alloc::string::String,
  #[prost(string, tag = "5")]
  pub tx_id: ::prost::alloc::string::String,
  #[prost(string, tag = "6")]
  pub encoded_claims: ::prost::alloc::string::String,
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
  pub payload: ::core::option::Option<OutputKind>,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OutputSignal {
  Close = 0,
  OpenBracket = 1,
  CloseBracket = 2,
}
#[doc = r" Generated client implementations."]
pub mod invocation_service_client {
  #![allow(unused_variables, dead_code, missing_docs)]
  use tonic::codegen::*;
  #[doc = " Interface exported by the server."]
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
    T::ResponseBody: Body + HttpBody + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
  {
    pub fn new(inner: T) -> Self {
      let inner = tonic::client::Grpc::new(inner);
      Self { inner }
    }
    pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
      let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
      Self { inner }
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
  impl<T: Clone> Clone for InvocationServiceClient<T> {
    fn clone(&self) -> Self {
      Self {
        inner: self.inner.clone(),
      }
    }
  }
  impl<T> std::fmt::Debug for InvocationServiceClient<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "InvocationServiceClient {{ ... }}")
    }
  }
}
#[doc = r" Generated server implementations."]
pub mod invocation_service_server {
  #![allow(unused_variables, dead_code, missing_docs)]
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
  }
  struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
  impl<T: InvocationService> InvocationServiceServer<T> {
    pub fn new(inner: T) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, None);
      Self { inner }
    }
    pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, Some(interceptor.into()));
      Self { inner }
    }
  }
  impl<T, B> Service<http::Request<B>> for InvocationServiceServer<T>
  where
    T: InvocationService,
    B: HttpBody + Send + Sync + 'static,
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
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1;
            let inner = inner.0;
            let method = InvokeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
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
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ListSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
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
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = StatsSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
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
              .body(tonic::body::BoxBody::empty())
              .unwrap(),
          )
        }),
      }
    }
  }
  impl<T: InvocationService> Clone for InvocationServiceServer<T> {
    fn clone(&self) -> Self {
      let inner = self.inner.clone();
      Self { inner }
    }
  }
  impl<T: InvocationService> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(self.0.clone(), self.1.clone())
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
