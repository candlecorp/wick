#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessagePayload {
  #[prost(enumeration = "PayloadKind", tag = "1")]
  pub kind: i32,
  #[prost(bytes = "vec", tag = "2")]
  pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entity {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(enumeration = "entity::EntityKind", tag = "2")]
  pub kind: i32,
}
/// Nested message and enum types in `Entity`.
pub mod entity {
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
  #[repr(i32)]
  pub enum EntityKind {
    Invalid = 0,
    Test = 1,
    Schematic = 2,
    Component = 3,
    Provider = 4,
    Port = 5,
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Invocation {
  #[prost(message, optional, tag = "1")]
  pub origin: ::core::option::Option<Entity>,
  #[prost(message, optional, tag = "2")]
  pub target: ::core::option::Option<Entity>,
  #[prost(message, optional, tag = "3")]
  pub msg: ::core::option::Option<MessagePayload>,
  #[prost(string, tag = "4")]
  pub id: ::prost::alloc::string::String,
  #[prost(string, tag = "5")]
  pub tx_id: ::prost::alloc::string::String,
  #[prost(string, tag = "6")]
  pub encoded_claims: ::prost::alloc::string::String,
  #[prost(string, tag = "7")]
  pub host_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Output {
  #[prost(string, tag = "1")]
  pub port: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub invocation_id: ::prost::alloc::string::String,
  #[prost(message, optional, tag = "3")]
  pub payload: ::core::option::Option<MessagePayload>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ack {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShutdownRequest {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PayloadKind {
  Invalid = 0,
  Test = 1,
  MessagePack = 2,
  MultiBytes = 3,
  Exception = 4,
  Error = 5,
}
#[doc = r" Generated client implementations."]
pub mod component_rpc_client {
  #![allow(unused_variables, dead_code, missing_docs)]
  use tonic::codegen::*;
  #[doc = " Interface exported by the server."]
  pub struct ComponentRpcClient<T> {
    inner: tonic::client::Grpc<T>,
  }
  impl ComponentRpcClient<tonic::transport::Channel> {
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
  impl<T> ComponentRpcClient<T>
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
      let path = http::uri::PathAndQuery::from_static("/vino.ComponentRpc/Invoke");
      self
        .inner
        .server_streaming(request.into_request(), path, codec)
        .await
    }
    pub async fn shutdown(
      &mut self,
      request: impl tonic::IntoRequest<super::ShutdownRequest>,
    ) -> Result<tonic::Response<super::Ack>, tonic::Status> {
      self.inner.ready().await.map_err(|e| {
        tonic::Status::new(
          tonic::Code::Unknown,
          format!("Service was not ready: {}", e.into()),
        )
      })?;
      let codec = tonic::codec::ProstCodec::default();
      let path = http::uri::PathAndQuery::from_static("/vino.ComponentRpc/Shutdown");
      self.inner.unary(request.into_request(), path, codec).await
    }
  }
  impl<T: Clone> Clone for ComponentRpcClient<T> {
    fn clone(&self) -> Self {
      Self {
        inner: self.inner.clone(),
      }
    }
  }
  impl<T> std::fmt::Debug for ComponentRpcClient<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "ComponentRpcClient {{ ... }}")
    }
  }
}
#[doc = r" Generated server implementations."]
pub mod component_rpc_server {
  #![allow(unused_variables, dead_code, missing_docs)]
  use tonic::codegen::*;
  #[doc = "Generated trait containing gRPC methods that should be implemented for use with ComponentRpcServer."]
  #[async_trait]
  pub trait ComponentRpc: Send + Sync + 'static {
    #[doc = "Server streaming response type for the Invoke method."]
    type InvokeStream: futures_core::Stream<Item = Result<super::Output, tonic::Status>>
      + Send
      + Sync
      + 'static;
    async fn invoke(
      &self,
      request: tonic::Request<super::Invocation>,
    ) -> Result<tonic::Response<Self::InvokeStream>, tonic::Status>;
    async fn shutdown(
      &self,
      request: tonic::Request<super::ShutdownRequest>,
    ) -> Result<tonic::Response<super::Ack>, tonic::Status>;
  }
  #[doc = " Interface exported by the server."]
  #[derive(Debug)]
  pub struct ComponentRpcServer<T: ComponentRpc> {
    inner: _Inner<T>,
  }
  struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
  impl<T: ComponentRpc> ComponentRpcServer<T> {
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
  impl<T, B> Service<http::Request<B>> for ComponentRpcServer<T>
  where
    T: ComponentRpc,
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
        "/vino.ComponentRpc/Invoke" => {
          #[allow(non_camel_case_types)]
          struct InvokeSvc<T: ComponentRpc>(pub Arc<T>);
          impl<T: ComponentRpc> tonic::server::ServerStreamingService<super::Invocation> for InvokeSvc<T> {
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
        "/vino.ComponentRpc/Shutdown" => {
          #[allow(non_camel_case_types)]
          struct ShutdownSvc<T: ComponentRpc>(pub Arc<T>);
          impl<T: ComponentRpc> tonic::server::UnaryService<super::ShutdownRequest> for ShutdownSvc<T> {
            type Response = super::Ack;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::ShutdownRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).shutdown(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ShutdownSvc(inner);
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
  impl<T: ComponentRpc> Clone for ComponentRpcServer<T> {
    fn clone(&self) -> Self {
      let inner = self.inner.clone();
      Self { inner }
    }
  }
  impl<T: ComponentRpc> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(self.0.clone(), self.1.clone())
    }
  }
  impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self.0)
    }
  }
  impl<T: ComponentRpc> tonic::transport::NamedService for ComponentRpcServer<T> {
    const NAME: &'static str = "vino.ComponentRpc";
  }
}
