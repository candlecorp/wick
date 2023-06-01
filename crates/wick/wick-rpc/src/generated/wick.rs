#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InvocationRequest {
  #[prost(oneof = "invocation_request::Data", tags = "1, 2")]
  pub data: ::core::option::Option<invocation_request::Data>,
}
/// Nested message and enum types in `InvocationRequest`.
pub mod invocation_request {
  #[allow(clippy::derive_partial_eq_without_eq)]
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Data {
    #[prost(message, tag = "1")]
    Invocation(super::Invocation),
    #[prost(message, tag = "2")]
    Packet(super::Packet),
  }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Invocation {
  #[prost(string, tag = "1")]
  pub origin: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub target: ::prost::alloc::string::String,
  #[prost(string, tag = "4")]
  pub id: ::prost::alloc::string::String,
  #[prost(string, tag = "5")]
  pub tx_id: ::prost::alloc::string::String,
  #[prost(message, optional, tag = "6")]
  pub inherent: ::core::option::Option<InherentData>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Packet {
  #[prost(message, optional, tag = "1")]
  pub metadata: ::core::option::Option<Metadata>,
  #[prost(oneof = "packet::Data", tags = "2, 3")]
  pub data: ::core::option::Option<packet::Data>,
}
/// Nested message and enum types in `Packet`.
pub mod packet {
  #[allow(clippy::derive_partial_eq_without_eq)]
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Data {
    #[prost(message, tag = "2")]
    Ok(super::Ok),
    #[prost(message, tag = "3")]
    Err(super::Err),
  }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
  #[prost(string, tag = "1")]
  pub port: ::prost::alloc::string::String,
  #[prost(uint32, tag = "2")]
  pub index: u32,
  #[prost(uint32, tag = "3")]
  pub flags: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ok {
  #[prost(oneof = "ok::Data", tags = "1, 3")]
  pub data: ::core::option::Option<ok::Data>,
}
/// Nested message and enum types in `Ok`.
pub mod ok {
  #[allow(clippy::derive_partial_eq_without_eq)]
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Data {
    #[prost(bytes, tag = "1")]
    Messagepack(::prost::alloc::vec::Vec<u8>),
    #[prost(string, tag = "3")]
    Json(::prost::alloc::string::String),
  }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Err {
  #[prost(string, tag = "1")]
  pub message: ::prost::alloc::string::String,
  #[prost(uint32, tag = "2")]
  pub code: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InherentData {
  #[prost(uint64, tag = "1")]
  pub seed: u64,
  #[prost(uint64, tag = "2")]
  pub timestamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResponse {
  #[prost(message, repeated, tag = "1")]
  pub components: ::prost::alloc::vec::Vec<ComponentSignature>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Operation {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(enumeration = "operation::OperationKind", tag = "2")]
  pub kind: i32,
  #[prost(message, repeated, tag = "3")]
  pub inputs: ::prost::alloc::vec::Vec<Field>,
  #[prost(message, repeated, tag = "4")]
  pub outputs: ::prost::alloc::vec::Vec<Field>,
}
/// Nested message and enum types in `Operation`.
pub mod operation {
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
  #[repr(i32)]
  pub enum OperationKind {
    Operation = 0,
    Schematic = 1,
  }
  impl OperationKind {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
      match self {
        OperationKind::Operation => "Operation",
        OperationKind::Schematic => "Schematic",
      }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
      match value {
        "Operation" => Some(Self::Operation),
        "Schematic" => Some(Self::Schematic),
        _ => None,
      }
    }
  }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Field {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(message, optional, tag = "2")]
  pub r#type: ::core::option::Option<TypeSignature>,
  #[prost(string, tag = "3")]
  pub description: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComponentSignature {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(uint32, tag = "2")]
  pub format: u32,
  #[prost(message, optional, tag = "8")]
  pub metadata: ::core::option::Option<ComponentMetadata>,
  #[prost(message, repeated, tag = "3")]
  pub operations: ::prost::alloc::vec::Vec<Operation>,
  #[prost(message, repeated, tag = "4")]
  pub types: ::prost::alloc::vec::Vec<TypeDefinition>,
  #[prost(message, repeated, tag = "5")]
  pub config: ::prost::alloc::vec::Vec<Field>,
  #[prost(message, repeated, tag = "7")]
  pub wellknown: ::prost::alloc::vec::Vec<WellKnownSchema>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComponentMetadata {
  #[prost(string, optional, tag = "2")]
  pub version: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TypeDefinition {
  #[prost(oneof = "type_definition::Type", tags = "1, 2")]
  pub r#type: ::core::option::Option<type_definition::Type>,
}
/// Nested message and enum types in `TypeDefinition`.
pub mod type_definition {
  #[allow(clippy::derive_partial_eq_without_eq)]
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Type {
    #[prost(message, tag = "1")]
    Struct(super::StructSignature),
    #[prost(message, tag = "2")]
    Enum(super::EnumSignature),
  }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WellKnownSchema {
  #[prost(string, repeated, tag = "1")]
  pub capabilities: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
  #[prost(string, tag = "2")]
  pub url: ::prost::alloc::string::String,
  #[prost(message, optional, tag = "3")]
  pub schema: ::core::option::Option<ComponentSignature>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatsResponse {
  #[prost(message, repeated, tag = "1")]
  pub stats: ::prost::alloc::vec::Vec<Statistic>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Statistic {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(uint32, tag = "2")]
  pub runs: u32,
  #[prost(uint32, tag = "3")]
  pub errors: u32,
  #[prost(message, optional, tag = "4")]
  pub execution_statistics: ::core::option::Option<DurationStatistics>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DurationStatistics {
  #[prost(uint64, tag = "1")]
  pub min: u64,
  #[prost(uint64, tag = "2")]
  pub max: u64,
  #[prost(uint64, tag = "3")]
  pub average: u64,
  #[prost(uint64, tag = "4")]
  pub total: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StructSignature {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(message, repeated, tag = "2")]
  pub fields: ::prost::alloc::vec::Vec<Field>,
  #[prost(string, tag = "3")]
  pub description: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnumSignature {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(message, repeated, tag = "2")]
  pub values: ::prost::alloc::vec::Vec<EnumVariant>,
  #[prost(string, tag = "3")]
  pub description: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnumVariant {
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  #[prost(uint32, optional, tag = "2")]
  pub index: ::core::option::Option<u32>,
  #[prost(string, optional, tag = "3")]
  pub value: ::core::option::Option<::prost::alloc::string::String>,
  #[prost(string, tag = "4")]
  pub description: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TypeSignature {
  #[prost(oneof = "type_signature::Signature", tags = "1, 2, 3, 4, 5, 6, 10, 11")]
  pub signature: ::core::option::Option<type_signature::Signature>,
}
/// Nested message and enum types in `TypeSignature`.
pub mod type_signature {
  #[allow(clippy::derive_partial_eq_without_eq)]
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Signature {
    #[prost(message, tag = "1")]
    Simple(super::SimpleType),
    #[prost(message, tag = "2")]
    Map(::prost::alloc::boxed::Box<super::MapType>),
    #[prost(message, tag = "3")]
    List(::prost::alloc::boxed::Box<super::InnerType>),
    #[prost(message, tag = "4")]
    Optional(::prost::alloc::boxed::Box<super::InnerType>),
    #[prost(message, tag = "5")]
    Ref(super::RefType),
    #[prost(message, tag = "6")]
    Link(super::LinkType),
    #[prost(message, tag = "10")]
    AnonymousStruct(super::AnonymousStruct),
    #[prost(string, tag = "11")]
    Custom(::prost::alloc::string::String),
  }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnonymousStruct {
  #[prost(message, repeated, tag = "1")]
  pub fields: ::prost::alloc::vec::Vec<Field>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleType {
  #[prost(enumeration = "simple_type::PrimitiveType", tag = "1")]
  pub r#type: i32,
}
/// Nested message and enum types in `SimpleType`.
pub mod simple_type {
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
  #[repr(i32)]
  pub enum PrimitiveType {
    I8 = 0,
    U8 = 1,
    I16 = 2,
    U16 = 3,
    I32 = 4,
    U32 = 5,
    I64 = 6,
    U64 = 7,
    F32 = 8,
    F64 = 9,
    Bool = 10,
    String = 11,
    Datetime = 12,
    Bytes = 13,
    Object = 15,
  }
  impl PrimitiveType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
      match self {
        PrimitiveType::I8 => "I8",
        PrimitiveType::U8 => "U8",
        PrimitiveType::I16 => "I16",
        PrimitiveType::U16 => "U16",
        PrimitiveType::I32 => "I32",
        PrimitiveType::U32 => "U32",
        PrimitiveType::I64 => "I64",
        PrimitiveType::U64 => "U64",
        PrimitiveType::F32 => "F32",
        PrimitiveType::F64 => "F64",
        PrimitiveType::Bool => "BOOL",
        PrimitiveType::String => "STRING",
        PrimitiveType::Datetime => "DATETIME",
        PrimitiveType::Bytes => "BYTES",
        PrimitiveType::Object => "OBJECT",
      }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
      match value {
        "I8" => Some(Self::I8),
        "U8" => Some(Self::U8),
        "I16" => Some(Self::I16),
        "U16" => Some(Self::U16),
        "I32" => Some(Self::I32),
        "U32" => Some(Self::U32),
        "I64" => Some(Self::I64),
        "U64" => Some(Self::U64),
        "F32" => Some(Self::F32),
        "F64" => Some(Self::F64),
        "BOOL" => Some(Self::Bool),
        "STRING" => Some(Self::String),
        "DATETIME" => Some(Self::Datetime),
        "BYTES" => Some(Self::Bytes),
        "OBJECT" => Some(Self::Object),
        _ => None,
      }
    }
  }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RefType {
  #[prost(string, tag = "1")]
  pub r#ref: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LinkType {
  #[prost(string, repeated, tag = "1")]
  pub schemas: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MapType {
  #[prost(message, optional, boxed, tag = "1")]
  pub key_type: ::core::option::Option<::prost::alloc::boxed::Box<TypeSignature>>,
  #[prost(message, optional, boxed, tag = "2")]
  pub value_type: ::core::option::Option<::prost::alloc::boxed::Box<TypeSignature>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InnerType {
  #[prost(message, optional, boxed, tag = "1")]
  pub r#type: ::core::option::Option<::prost::alloc::boxed::Box<TypeSignature>>,
}
/// Generated client implementations.
pub mod invocation_service_client {
  #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
  use tonic::codegen::http::Uri;
  use tonic::codegen::*;
  #[derive(Debug, Clone)]
  pub struct InvocationServiceClient<T> {
    inner: tonic::client::Grpc<T>,
  }
  impl InvocationServiceClient<tonic::transport::Channel> {
    /// Attempt to create a new client by connecting to a given endpoint.
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
      D: TryInto<tonic::transport::Endpoint>,
      D::Error: Into<StdError>,
    {
      let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
      Ok(Self::new(conn))
    }
  }
  impl<T> InvocationServiceClient<T>
  where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
  {
    pub fn new(inner: T) -> Self {
      let inner = tonic::client::Grpc::new(inner);
      Self { inner }
    }
    pub fn with_origin(inner: T, origin: Uri) -> Self {
      let inner = tonic::client::Grpc::with_origin(inner, origin);
      Self { inner }
    }
    pub fn with_interceptor<F>(inner: T, interceptor: F) -> InvocationServiceClient<InterceptedService<T, F>>
    where
      F: tonic::service::Interceptor,
      T::ResponseBody: Default,
      T: tonic::codegen::Service<
        http::Request<tonic::body::BoxBody>,
        Response = http::Response<<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody>,
      >,
      <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error: Into<StdError> + Send + Sync,
    {
      InvocationServiceClient::new(InterceptedService::new(inner, interceptor))
    }
    /// Compress requests with the given encoding.
    ///
    /// This requires the server to support it otherwise it might respond with an
    /// error.
    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
      self.inner = self.inner.send_compressed(encoding);
      self
    }
    /// Enable decompressing responses.
    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
      self.inner = self.inner.accept_compressed(encoding);
      self
    }
    /// Limits the maximum size of a decoded message.
    ///
    /// Default: `4MB`
    #[must_use]
    pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
      self.inner = self.inner.max_decoding_message_size(limit);
      self
    }
    /// Limits the maximum size of an encoded message.
    ///
    /// Default: `usize::MAX`
    #[must_use]
    pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
      self.inner = self.inner.max_encoding_message_size(limit);
      self
    }
    pub async fn invoke(
      &mut self,
      request: impl tonic::IntoStreamingRequest<Message = super::InvocationRequest>,
    ) -> std::result::Result<tonic::Response<tonic::codec::Streaming<super::Packet>>, tonic::Status> {
      self
        .inner
        .ready()
        .await
        .map_err(|e| tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into())))?;
      let codec = tonic::codec::ProstCodec::default();
      let path = http::uri::PathAndQuery::from_static("/wick.InvocationService/Invoke");
      let mut req = request.into_streaming_request();
      req
        .extensions_mut()
        .insert(GrpcMethod::new("wick.InvocationService", "Invoke"));
      self.inner.streaming(req, path, codec).await
    }
    pub async fn list(
      &mut self,
      request: impl tonic::IntoRequest<super::ListRequest>,
    ) -> std::result::Result<tonic::Response<super::ListResponse>, tonic::Status> {
      self
        .inner
        .ready()
        .await
        .map_err(|e| tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into())))?;
      let codec = tonic::codec::ProstCodec::default();
      let path = http::uri::PathAndQuery::from_static("/wick.InvocationService/List");
      let mut req = request.into_request();
      req
        .extensions_mut()
        .insert(GrpcMethod::new("wick.InvocationService", "List"));
      self.inner.unary(req, path, codec).await
    }
    pub async fn stats(
      &mut self,
      request: impl tonic::IntoRequest<super::StatsRequest>,
    ) -> std::result::Result<tonic::Response<super::StatsResponse>, tonic::Status> {
      self
        .inner
        .ready()
        .await
        .map_err(|e| tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into())))?;
      let codec = tonic::codec::ProstCodec::default();
      let path = http::uri::PathAndQuery::from_static("/wick.InvocationService/Stats");
      let mut req = request.into_request();
      req
        .extensions_mut()
        .insert(GrpcMethod::new("wick.InvocationService", "Stats"));
      self.inner.unary(req, path, codec).await
    }
  }
}
/// Generated server implementations.
pub mod invocation_service_server {
  #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
  use tonic::codegen::*;
  /// Generated trait containing gRPC methods that should be implemented for use with InvocationServiceServer.
  #[async_trait]
  pub trait InvocationService: Send + Sync + 'static {
    /// Server streaming response type for the Invoke method.
    type InvokeStream: futures_core::Stream<Item = std::result::Result<super::Packet, tonic::Status>> + Send + 'static;
    async fn invoke(
      &self,
      request: tonic::Request<tonic::Streaming<super::InvocationRequest>>,
    ) -> std::result::Result<tonic::Response<Self::InvokeStream>, tonic::Status>;
    async fn list(
      &self,
      request: tonic::Request<super::ListRequest>,
    ) -> std::result::Result<tonic::Response<super::ListResponse>, tonic::Status>;
    async fn stats(
      &self,
      request: tonic::Request<super::StatsRequest>,
    ) -> std::result::Result<tonic::Response<super::StatsResponse>, tonic::Status>;
  }
  #[derive(Debug)]
  pub struct InvocationServiceServer<T: InvocationService> {
    inner: _Inner<T>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
    max_decoding_message_size: Option<usize>,
    max_encoding_message_size: Option<usize>,
  }
  struct _Inner<T>(Arc<T>);
  impl<T: InvocationService> InvocationServiceServer<T> {
    pub fn new(inner: T) -> Self {
      Self::from_arc(Arc::new(inner))
    }
    pub fn from_arc(inner: Arc<T>) -> Self {
      let inner = _Inner(inner);
      Self {
        inner,
        accept_compression_encodings: Default::default(),
        send_compression_encodings: Default::default(),
        max_decoding_message_size: None,
        max_encoding_message_size: None,
      }
    }
    pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
    where
      F: tonic::service::Interceptor,
    {
      InterceptedService::new(Self::new(inner), interceptor)
    }
    /// Enable decompressing requests with the given encoding.
    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
      self.accept_compression_encodings.enable(encoding);
      self
    }
    /// Compress responses with the given encoding, if the client supports it.
    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
      self.send_compression_encodings.enable(encoding);
      self
    }
    /// Limits the maximum size of a decoded message.
    ///
    /// Default: `4MB`
    #[must_use]
    pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
      self.max_decoding_message_size = Some(limit);
      self
    }
    /// Limits the maximum size of an encoded message.
    ///
    /// Default: `usize::MAX`
    #[must_use]
    pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
      self.max_encoding_message_size = Some(limit);
      self
    }
  }
  impl<T, B> tonic::codegen::Service<http::Request<B>> for InvocationServiceServer<T>
  where
    T: InvocationService,
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
  {
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = std::convert::Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: http::Request<B>) -> Self::Future {
      let inner = self.inner.clone();
      match req.uri().path() {
        "/wick.InvocationService/Invoke" => {
          #[allow(non_camel_case_types)]
          struct InvokeSvc<T: InvocationService>(pub Arc<T>);
          impl<T: InvocationService> tonic::server::StreamingService<super::InvocationRequest> for InvokeSvc<T> {
            type Response = super::Packet;
            type ResponseStream = T::InvokeStream;
            type Future = BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<tonic::Streaming<super::InvocationRequest>>) -> Self::Future {
              let inner = Arc::clone(&self.0);
              let fut = async move { (*inner).invoke(request).await };
              Box::pin(fut)
            }
          }
          let accept_compression_encodings = self.accept_compression_encodings;
          let send_compression_encodings = self.send_compression_encodings;
          let max_decoding_message_size = self.max_decoding_message_size;
          let max_encoding_message_size = self.max_encoding_message_size;
          let inner = self.inner.clone();
          let fut = async move {
            let inner = inner.0;
            let method = InvokeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = tonic::server::Grpc::new(codec)
              .apply_compression_config(accept_compression_encodings, send_compression_encodings)
              .apply_max_message_size_config(max_decoding_message_size, max_encoding_message_size);
            let res = grpc.streaming(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/wick.InvocationService/List" => {
          #[allow(non_camel_case_types)]
          struct ListSvc<T: InvocationService>(pub Arc<T>);
          impl<T: InvocationService> tonic::server::UnaryService<super::ListRequest> for ListSvc<T> {
            type Response = super::ListResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::ListRequest>) -> Self::Future {
              let inner = Arc::clone(&self.0);
              let fut = async move { (*inner).list(request).await };
              Box::pin(fut)
            }
          }
          let accept_compression_encodings = self.accept_compression_encodings;
          let send_compression_encodings = self.send_compression_encodings;
          let max_decoding_message_size = self.max_decoding_message_size;
          let max_encoding_message_size = self.max_encoding_message_size;
          let inner = self.inner.clone();
          let fut = async move {
            let inner = inner.0;
            let method = ListSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = tonic::server::Grpc::new(codec)
              .apply_compression_config(accept_compression_encodings, send_compression_encodings)
              .apply_max_message_size_config(max_decoding_message_size, max_encoding_message_size);
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/wick.InvocationService/Stats" => {
          #[allow(non_camel_case_types)]
          struct StatsSvc<T: InvocationService>(pub Arc<T>);
          impl<T: InvocationService> tonic::server::UnaryService<super::StatsRequest> for StatsSvc<T> {
            type Response = super::StatsResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::StatsRequest>) -> Self::Future {
              let inner = Arc::clone(&self.0);
              let fut = async move { (*inner).stats(request).await };
              Box::pin(fut)
            }
          }
          let accept_compression_encodings = self.accept_compression_encodings;
          let send_compression_encodings = self.send_compression_encodings;
          let max_decoding_message_size = self.max_decoding_message_size;
          let max_encoding_message_size = self.max_encoding_message_size;
          let inner = self.inner.clone();
          let fut = async move {
            let inner = inner.0;
            let method = StatsSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = tonic::server::Grpc::new(codec)
              .apply_compression_config(accept_compression_encodings, send_compression_encodings)
              .apply_max_message_size_config(max_decoding_message_size, max_encoding_message_size);
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
        max_decoding_message_size: self.max_decoding_message_size,
        max_encoding_message_size: self.max_encoding_message_size,
      }
    }
  }
  impl<T: InvocationService> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(Arc::clone(&self.0))
    }
  }
  impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self.0)
    }
  }
  impl<T: InvocationService> tonic::server::NamedService for InvocationServiceServer<T> {
    const NAME: &'static str = "wick.InvocationService";
  }
}
