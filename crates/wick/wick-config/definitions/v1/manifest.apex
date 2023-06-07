namespace "wick::v1"

"""
Root configuration that can be any one of the possible Wick configuration formats.
"""
union WickConfig = AppConfiguration | ComponentConfiguration | TypesConfiguration | TestConfiguration

"A reference to a location on disk, URL, or registry."
alias LocationReference = string @skip

"""
A liquid JSON value. Liquid-JSON is a way of using Liquid templates in structured JSON-like data.

See [liquid-json](https://crates.io/crates/liquid-json) for more information.
"""
alias LiquidJsonValue = string @skip

"A glob pattern, e.g. **/*.html"
alias Glob = string @skip

"Configuration for a standalone Wick application."
type AppConfiguration @default @tagged("wick/app@v1") {
  "The application's name."
  name: string,

  "Associated metadata for this component."
  metadata: Metadata?,

  "Details about the package for this application."
  package: PackageDefinition?,

  "Resources and configuration that the application and its components can access."
  resources: [ResourceBinding],

  "Components that to import and make available to the application."
  import: [ImportBinding],

  "Triggers to load and instantiate to drive the application's behavior."
  triggers: [TriggerDefinition],
}

"Metadata to associate with an artifact."
type Metadata {
  "The version of the artifact."
  version: string,
  "A list of the authors."
  authors: [string],
  "A list of any vendors associated with the artifact."
  vendors: [string],
  "A short description."
  description: string?,
  "Where to find documentation."
  documentation: string?,
  "The license(s) for the artifact."
  licenses: [string],
  "An icon to associate with the artifact."
  icon: LocationReference?,
}

"Configuration for packaging and publishing Wick configurations."
type PackageDefinition @default {
  "The list of files and folders to be included with the package."
  files: [Glob]
  "Configuration for publishing the package to a registry."
  registry: RegistryDefinition?
}

type RegistryDefinition @default {
  "The registry to publish to, e.g. registry.candle.dev"
  host: string = "registry.candle.dev"  @alias("registry")
  "The namespace on the registry. e.g.: [*your username*]"
  namespace: string
}

"An identifier bound to a resource."
type ResourceBinding {
  "The name of the binding."
  name: string @required

  "The resource to bind to."
  resource: ResourceDefinition @required
}

"An identifier bound to an imported component or type manifest."
type ImportBinding {
  "The name of the binding."
  name: string @required

  "The import to bind to."
  component: ImportDefinition @required
}

"The possible types of resources. Resources are system-level resources and sensitive configuration."
union ResourceDefinition = TcpPort | UdpPort | Url | Volume

"A TCP port to bind to."
type TcpPort @tagged("wick/resource/tcpport@v1") {
  "The port to bind to."
  port: u16
  "The address to bind to."
  address: string
}

"A UDP port to bind to."
type UdpPort @tagged("wick/resource/udpport@v1") {
  "The port to bind to."
  port: u16
  "The address to bind to."
  address: string
}

"A filesystem or network volume resource."
type Volume @tagged("wick/resource/volume@v1") {
  "The path."
  path: string @required
}

"A URL configured as a resource."
type Url @tagged("wick/resource/url@v1") {
  "The url string."
  url: string @required
}

"Triggers that operate off events and translate environment data to components. Triggers are the way that Wick handles standard use cases and translates them into the component world."
union TriggerDefinition = CliTrigger | HttpTrigger | TimeTrigger

"A trigger that runs when an application is called via the command line."
type CliTrigger @tagged("wick/trigger/cli@v1") {
  "The operation that will act as the main entrypoint for this trigger."
  operation: ComponentOperationExpression @required @shortform
}

"A trigger that runs on a schedule similar to cron."
type TimeTrigger @tagged("wick/trigger/time@v1") {
  "The schedule to run the trigger with."
  schedule: Schedule @required

  "The operation to execute on the schedule."
  operation: ComponentOperationExpression @required @shortform

  "Values passed to the operation as inputs"
  payload: [OperationInput] @required
}

"Input to use when calling an operation"
type OperationInput {
  "The name of the input."
  name: string @required

  "The value to pass."
  value: any @required
}


"The schedule to run the Time trigger with."
type Schedule {
  "Schedule in cron format with second precision. See [cron.help](https://cron.help) for more information."
  cron: string @required

  "repeat `n` times. Use `0` to repeat indefinitely"
  repeat: u16 = 0
}

"A reference to an operation. This type can be shortened to <code>component_id::operation_name</code> with the short-form syntax."
type ComponentOperationExpression @shortform {
  "The component that exports the operation."
  component: ComponentDefinition @required @shortform
  "The operation name."
  name: string @required
  "Configuration to pass to this operation on invocation."
  with: {string: any}?,
}

"An HTTP server that delegates to HTTP routers on every request."
type HttpTrigger @tagged("wick/trigger/http@v1") {
  "The TcpPort resource to listen on for connections."
  resource: string @required

  "The router to handle incoming requests"
  routers: [HttpRouter]
}

"The types of routers that can be configured on the HttpTrigger."
union HttpRouter = RawRouter | RestRouter | StaticRouter | ProxyRouter

"A router that proxies to the configured URL when the path matches."
type ProxyRouter @tagged("wick/router/proxy@v1") {
  "The path that this router will trigger for."
  path: string @required

  "Middleware operations for this router."
  middleware: Middleware?

  "The URL resource to proxy to."
  url: string @required

  "Whether or not to strip the router's path from the proxied request."
  strip_path: bool
}

"A router that can be configured to delegate to specific operations on a per-route, per-method basis."
type RestRouter @tagged("wick/router/rest@v1") {
  "The path that this router will trigger for."
  path: string @required

  "Additional tools and services to enable."
  tools: Tools?

  "Middleware operations for this router."
  middleware: Middleware?

  "The routes to serve and operations that handle them."
  routes: [Route]

  "Information about the router to use when generating documentation and other tools."
  info: Info?
}

"A route to serve and the operation that handles it."
type Route {
  "The path to serve this route from. See [URI documentation](/docs/configuration/uri) for more information on specifying query and path parameters."
  uri: string @required
  "The operation that will act as the main entrypoint for this route."
  operation: ComponentOperationExpression @required @shortform
  "The HTTP methods to serve this route for."
  methods: [string]
  "The name of the route, used for documentation and tooling."
  name: string?
  "A short description of the route."
  description: string?
  "A longer description of the route."
  summary: string?
}

"Additional tools and services to enable."
type Tools {
  "The path to serve the OpenAPI spec from"
  openapi: string?
}

"Information about the router to use when generating documentation and other tools."
type Info {
  "The title of the API."
  title:        string?
  "A short description of the API."
	description:  string?
  "The terms of service for the API."
	tos:          string?
  "The contact information for the API."
	contact:      Contact?
  "The license information for the API."
	license:      License?
  "The version of the API."
	version:      string?
  "The URL to the API's terms of service."
  documentation: Documentation?
}

"Information about where and how the API is documented."
type Documentation {
  "The URL to the API's documentation."
  url:         string?

  "A short description of the documentation."
  description: string?
}

"Any licensing information for the API."
type License {
  "The name of the license."
	name: string?
  "The URL to the license."
	url:  string?
}

"Contact information to expose for the API."
type Contact {
  "The name of the contact."
	name: string?
  "The URL to the contact."
	url:  string?
  "The email address of the contact."
	email:  string?
}

"A router that serves static files."
type StaticRouter @tagged("wick/router/static@v1") {
  "The path that this router will trigger for."
  path: string @required

  "Middleware operations for this router."
  middleware: Middleware?

  "The volume to serve static files from."
  volume: string @required

  "Fallback path (relative to volume `resource`) for files to serve in case of a 404. Useful for SPA's. if volume resource is: /www and fallback: index.html, then a 404 will serve /www/index.html"
  fallback: string?
}

"A router that delegates all requests to the configured operation, optionally encoding/decoding based on the specified codec."
type RawRouter @tagged("wick/router/raw@v1") {
  "The path that this router will trigger for."
  path: string @required

  "Middleware operations for this router."
  middleware: Middleware?

  "The codec to use when encoding/decoding data."
  codec: Codec?

  "The operation that handles HTTP requests."
  operation: ComponentOperationExpression @required @shortform
}

"Request and response operations that run before and after the main operation."
type Middleware {
  "The middleware to apply to requests."
  request: [ComponentOperationExpression]
  "The middleware to apply to responses."
  response: [ComponentOperationExpression]
}

"A type definition for a Wick Components and Operations"
type TypesConfiguration @tagged("wick/types@v1") {
  "The name of this type."
  name: string?

  "Associated metadata for this type."
  metadata: Metadata?,

  "Additional types to export and make available to the type."
  types: [TypeDefinition]

  "A list of operation signatures."
  operations: [OperationDefinition]

  "Details about the package for this types."
  package: PackageDefinition?,
}

"A configuration for a Wick Component"
type TestConfiguration @tagged("wick/tests@v1") {
  "The name of this component."
  name: string?

  "Configuration used to instantiate this component."
  with: {string: any}?

  "Unit tests to run against components and operations."
  cases: [TestDefinition]
}


"A configuration for a Wick Component"
type ComponentConfiguration @tagged("wick/component@v1") {
  "The name of the component."
  name: string?

  "Associated metadata for this component."
  metadata: Metadata?

  "Details about the package for this component."
  package: PackageDefinition?,

  "Configuration for when wick hosts this component as a service."
  host: HostConfig?

  "Resources that the component can access."
  resources: [ResourceBinding]

  "Components or types to import into this component's scope."
  import: [ImportBinding]

  "Additional types to export and make available to the component."
  types: [TypeDefinition]

  "Interfaces the component requires to operate."
  requires: [InterfaceBinding]

  "Configuration specific to different kinds of components."
  component: ComponentKind @required

  "Assertions that can be run against the component to validate its behavior."
  tests: [TestConfiguration]
}

"An interface bound to an ID. Used in the require/provide relationship between components."
type InterfaceBinding {
  "The name of the interface."
  name: string @required

  "The interface to bind to."
  interface: InterfaceDefinition @required
}

"A interface definition. Used as a signature that components can require or provide."
type InterfaceDefinition {
  "Types used by the interface's operations"
  types: [TypeDefinition]

  "A list of operations defined by this interface."
  operations: [OperationDefinition]
}

"A component made from connectiong other components."
type CompositeComponentConfiguration @tagged("wick/component/composite@v1") {
  "Configuration necessary to provide when instantiating the component."
  with: [Field]

  "A list of operations exposed by the Composite component."
  operations: [CompositeOperationDefinition]
}

"A component whose implementation is a WasmRS WebAssembly module."
type WasmComponentConfiguration @tagged("wick/component/wasmrs@v1") {
  "The path or OCI reference to the WebAssembly module"
  ref: LocationReference @rename("reference") @required

  "Configuration necessary to provide when instantiating the component."
  with: [Field]

  "A list of operations implemented by the WebAssembly module."
  operations: [OperationDefinition]
}

"Root component types. These are the components that can be instantiated and run."
union ComponentKind = WasmComponentConfiguration | CompositeComponentConfiguration | SqlComponent | HttpClientComponent

"Types of possible imports."
union ImportDefinition = TypesComponent | ManifestComponent | SqlComponent | HttpClientComponent

"Component types used when referencing operations or linking components."
union ComponentDefinition = GrpcUrlComponent | ManifestComponent | ComponentReference | SqlComponent | HttpClientComponent

"A types configuration to import into this component's scope."
type TypesComponent @tagged("wick/component/types@v1") {
  "The URL (and optional tag) or local file path to find the types manifest."
  ref: LocationReference @rename("reference")  @required

  "The types to import from the manifest."
  types: [string]
}

"A reference to a component in the application's scope."
type ComponentReference  @tagged("wick/component/reference@v1") {
  "The id of the referenced component."
  id: string  @required
}

"Host configuration options."
type HostConfig @default {
  "Whether or not to allow the `:latest` tag on remote artifacts."
  allow_latest: bool,

  "A list of registries to connect to insecurely (over HTTP vs HTTPS)."
  insecure_registries: [string],

  "The timeout for network requests (in ms)."
  timeout: u64 = 5000,

  "Configuration for the GRPC server."
  rpc: HttpConfig?
}

"Configuration for the GRPC service."
type HttpConfig {
  "Enable/disable the server."
  enabled: bool,

  "The port to bind to."
  port: u16?,

  "The address to bind to."
  address: string?

  "Path to pem file for TLS."
  pem: LocationReference?,

  "Path to key file for TLS."
  key: LocationReference?,

  "Path to CA file."
  ca: LocationReference?,
}

"A component hosted as an independent microservice."
type GrpcUrlComponent @tagged("wick/component/grpc@v1") {
  "The GRPC URL to connect to."
  url: string @required

  "Any configuration necessary for the component."
  with: {string: any}?,
}

"A native component that can be extracted and run as a microservice."
type ManifestComponent @tagged("wick/component/manifest@v1") {
  "The URL (and optional tag) or local file path to find the manifest."
  ref: LocationReference @rename("reference")  @required

  "Any configuration necessary for the component."
  with: {string: any}?,

  "External components to provide to the referenced component."
  provide: {string:string}
}

"Composite operations are operations whose implementations come from connecting other operations into a flow or series of pipelines."
type CompositeOperationDefinition {
  "The name of the operation."
  name: string,

  "Any configuration required by the operation."
  with: [Field],

  "Types of the inputs to the operation."
  inputs: [Field],

  "Types of the outputs to the operation."
  outputs: [Field],

  "A map of IDs to specific operations."
  uses: [OperationInstance]

  "A list of connections from operation to operation."
  flow: [FlowExpression],

  "Additional `CompositeOperationDefinition`s to define as children."
  operations: [CompositeOperationDefinition]
}

"A flow operation, i.e. a connection from one operation's outputs to another's inputs."
union FlowExpression @shortform @untagged = ConnectionDefinition | BlockExpression

"A list of FlowExpressions. Typically used only when expanding a shortform `FlowExpression` into multiple `FlowExpression`s."
type BlockExpression  {
  expressions: [FlowExpression] @required
}

"A connection between Operations and their ports. This can be specified in short-form syntax."
type ConnectionDefinition @into_string {
  "An upstream operation's output."
  from: ConnectionTargetDefinition @required

  "A downstream operation's input."
  to: ConnectionTargetDefinition @required
}

"A connection target e.g. a specific input or output on an operation instance. This can be specified in shortform syntax."
type ConnectionTargetDefinition {
  "The instance ID of the component operation."
  instance: string @required,

  "The operation's input or output (depending on to/from)."
  port: string @required,

  "The default value to provide on this connection in the event of an error."
  data: {string: any}?,
}

"An operation name and its input and output signatures"
type OperationDefinition {
  "The name of the operation."
  name: string,

  "Any configuration required by the operation."
  with: [Field],

  "Types of the inputs to the operation."
  inputs: [Field],

  "Types of the outputs to the operation."
  outputs: [Field],
}

"Field definition with a name and type signature"
type Field {
  "The name of the field."
  name: string @required
  "The type signature of the field."
  type: TypeSignature @rename("ty") @required
  "The description of the field."
  description: string?,
}

union TypeSignature @into_string @shortform @shortformonly = I8 | I16 | I32 | I64 | U8 | U16 | U32 | U64 | F32 | F64 | Bool | StringType | Optional | Datetime | Bytes | Custom | List | Map | Object

type I8 @nameonly("i8") {}
type I16 @nameonly("i16") {}
type I32 @nameonly("i32") {}
type I64 @nameonly("i64") {}
type U8 @nameonly("u8") {}
type U16 @nameonly("u16") {}
type U32 @nameonly("u32") {}
type U64 @nameonly("u64") {}
type F32 @nameonly("f32") {}
type F64 @nameonly("f64") {}
type Bool @nameonly("i8") {}
type StringType @nameonly("string") {}
type Datetime @nameonly("datetime") {}
type Bytes @nameonly("bytes") {}
type Custom {
  "The name of the custom type."
  name: string
}
type Optional  {
  type: TypeSignature @rename("ty") @required @recursive
}
type List {
  type: TypeSignature @rename("ty") @required @recursive
}
type Map {
  key: TypeSignature @required @recursive
  value: TypeSignature @required @recursive
}
type Object @nameonly("object") {}

"A Struct or Enum type definition."
union TypeDefinition = StructSignature | EnumSignature

"A struct definition of named fields and types."
type StructSignature @tagged("wick/type/struct@v1") {
  "The name of the struct."
   name: string,

  "The fields in this struct."
   fields: [Field],

  "The description of the struct."
  description: string?,
}

"An enum definition of named variants."
type EnumSignature @tagged("wick/type/enum@v1") {
  "The name of the enum."
  name: string,

  "The variants in the enum."
  variants: [EnumVariant],

  "The description of the enum."
  description: string?,
}

"An enum variant."
type EnumVariant {
  "The name of the variant."
  name: string,

  "The index of the variant."
  index: u32?,

  "The optional value of the variant."
  value: string?,

  "A description of the variant."
  description: string?,
}

"An identifier bound to a component's operation."
type OperationInstance {
  "The name of the binding."
  name: string @required

  "The operation to bind to."
  operation: ComponentOperationExpression @required @shortform

  "Data to associate with the reference, if any."
  with: {string: any}?,
}

"A test case for a component's operation."
type TestDefinition {
  "The name of the test."
  name: string @required

  "The operaton to test."
  operation: string @required

  "Inherent data to use for the test."
  inherent: InherentData?

  "The configuration for the operation, if any."
  with: {string: any}?

  "The inputs to the test."
  inputs: [PacketData] @alias("input")

  "The expected outputs of the operation."
  outputs: [PacketData] @alias("output")
}

"Data inherent to all invocations."
type InherentData {
  "A random seed, i.e. to initialize a random number generator."
  seed: u64?

  "A timestamp."
  timestamp: u64?
}

"Either a success packet or an error packet."
union PacketData @untagged = SuccessPacket | ErrorPacket

"A simplified representation of a Wick data packet & payload, used when writing tests."
type SuccessPacket {
  "The name of the input or output this packet is going to or coming from."
  name: string @required

  "Any flags set on the packet."
  flags: PacketFlags?

  "The data to send."
  value: any? @skip_env @alias("data")
}

type ErrorPacket {
  "The name of the input or output this packet is going to or coming from."
  name: string @required

  "Any flags set on the packet."
  flags: PacketFlags?

  "The error message."
  error: string @required
}

"Flags set on a packet."
type PacketFlags {
  "Indicates the port should be considered closed."
  done: bool

  "Indicates the opening of a new substream context within the parent stream."
  open: bool

  "Indicates the closing of a substream context within the parent stream."
  close: bool
}

"A dynamic component whose operations are SQL queries to a database."
type SqlComponent @tagged("wick/component/sql@v1") {
  "The connect string URL resource for the database."
  resource: string

  "Whether or not to use TLS."
  tls: bool

  "A list of operations to expose on this component."
  operations: [SqlOperationDefinition]
}

"A dynamic operation whose implementation is a SQL query."
type SqlOperationDefinition {
  "The name of the operation."
  name: string

  "Any configuration required by the operation."
  with: [Field],

  "Types of the inputs to the operation."
  inputs: [Field],

  "Types of the outputs to the operation."
  outputs: [Field],

  "The query to execute."
  query: string @required

  "The positional arguments to the query, defined as a list of input names."
  arguments: [string]
}


"A component whose operations are HTTP requests."
type HttpClientComponent @tagged("wick/component/http@v1") {
  "The URL base to use."
  resource: string

  "The codec to use when encoding/decoding data. Can be overridden by individual operations."
  codec: Codec?

  "A list of operations to expose on this component."
  operations: [HttpClientOperationDefinition]
}

"A dynamic operation whose implementation is an HTTP request. The outputs of HttpClientOperationDefinition are always `response` & `body`"
type HttpClientOperationDefinition {
  "The name of the operation."
  name: string

  "Any configuration required by the operation."
  with: [Field],

  "Types of the inputs to the operation."
  inputs: [Field],

  "The HTTP method to use."
  method: HttpMethod

  "The codec to use when encoding/decoding data."
  codec: Codec?

  "Any headers to add to the request."
  headers: {string: [string]}?

  "The body to send, processed as a structured JSON liquid template."
  body: LiquidJsonValue?

  "The path to append to our base URL, processed as a liquid template with each input as part of the template data."
  path: string
}

"Codec to use when encoding/decoding data."
enum Codec {
  "JSON data"
  Json = 0 as "json",

  "Raw"
  Raw = 1 as "raw",

  "Form Data"
  FormData = 2 as "form-data",
}

"Supported HTTP methods"
enum HttpMethod {
  "GET method"
  Get = 0 as "get",

  "POST method"
  Post = 1 as "post",

  "PUT method"
  Put = 2 as "put",

  "DELETE method"
  Delete = 3 as "delete",
}
