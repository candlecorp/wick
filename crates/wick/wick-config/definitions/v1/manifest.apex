namespace "wick-config::v1"

"Configuration for Wick applications and components."
union WickConfig = AppConfiguration | ComponentConfiguration | TypesConfiguration | TestConfiguration

"A reference to a location on disk, URL, or registry."
alias LocationReference = string @skip

"The Application configuration defines a standalone Wick application."
type AppConfiguration @default @tagged("wick/app@v1") {
  "Associated metadata for this component."
  metadata: Metadata?,

  "The application's name."
  name: string,

  "Configuration that controls how this application runs within a host."
  host: HostConfig,

  "Components to import into the application's scope."
  import: [ComponentBinding],

  "Resources that the application can access."
  resources: [ResourceBinding],

  "Configured triggers that drive the application's behavior."
  triggers: [TriggerDefinition],
}

"Metadata for the component or application."
type Metadata {
  "The version of the component or application."
  version: string,
  "The authors of the component or application."
  authors: [string],
  "Any vendors associated with the component or application."
  vendors: [string],
  "A short description of the component or application."
  description: string?,
  "Where to find documentation for the component or application."
  documentation: string?,
  "The license(s) for the component or application."
  licenses: [string],
  "The icon for the component or application."
  icon: LocationReference?,
}

"The possible types of resources."
union TriggerDefinition = CliTrigger | HttpTrigger | TimeTrigger

"A trigger called with a CLI context."
type CliTrigger @tagged("wick/trigger/cli@v1") {
  "The operation that will act as the main entrypoint for this trigger."
  operation: ComponentOperationExpression @required
  "The component that provides additional logic."
  app: ComponentDefinition? @shortform
}

"A trigger called with a Time context."
type TimeTrigger @tagged("wick/trigger/time@v1") {
  schedule: Schedule @required
  "The operation that will act as the main entrypoint for this trigger."
  operation: ComponentOperationExpression @required
  "Values passed to the operation as inputs"
  payload: [OperationInput] @required
}

type OperationInput {
  "The name of the operation parameter."
  name: string @required
  "The value to pass to the operation parameter."
  value: any @required
}

type Schedule {
  "schedule in cron format with second precision"
  cron: string @required
  "repeat n times, 0 means forever"
  repeat: u16 = 0
}

"A reference to an operation with an explicit component definition."
type ComponentOperationExpression {
  "The component that exports the operation."
  component: ComponentDefinition @required @shortform
  "The operation to call."
  name: string @required
}

"An HTTP server that delegates to HTTP routers upon requests."
type HttpTrigger @tagged("wick/trigger/http@v1") {
  "The TcpPort reference to listen on for connections."
  resource: string

  "The HttpRouters that should handle incoming requests"
  routers: [HttpRouter]
}

union HttpRouter = RawRouter | RestRouter

type RestRouter @tagged("wick/router/rest@v1") {
  "The path to start serving this router from."
  path: string @required

  "The component to expose as a Rest API."
  component: ComponentDefinition @required
}

"A RawHttpRouter delegates raw requests and bodies to operations based on the request path."
type RawRouter @tagged("wick/router/raw@v1") {
  "The path to start serving this router from."
  path: string @required

  "The operation that handles HTTP requests."
  operation: ComponentOperationExpression @required
}

"The possible types of resources."
union ResourceDefinition = TcpPort | UdpPort

"A WebAssembly component."
type TcpPort @tagged("wick/resource/tcpport@v1") {
  "The port to bind to."
  port: u16
  "The address to bind to."
  address: string
}

"A WebAssembly component."
type UdpPort @tagged("wick/resource/udpport@v1") {
  "The port to bind to."
  port: u16
  "The address to bind to."
  address: string
}


"A configuration for a Wick Component"
type TypesConfiguration @tagged("wick/types@v1") {
  "The name of this component."
  name: string?

  "Additional types to export and make available to the component."
  types: [TypeDefinition]
}

"A configuration for a Wick Component"
type TestConfiguration @tagged("wick/tests@v1") {
  "The name of this component."
  name: string?

  "Unit tests to run against components and operations."
  tests: [TestDefinition]
}


"A configuration for a Wick Component"
type ComponentConfiguration @tagged("wick/component@v1") {
  "The name of this component."
  name: string?

  "Associated metadata for this component."
  metadata: Metadata?

  "Configuration for when wick hosts this component as a service."
  host: HostConfig

  "The labels and values that apply to this manifest."
  labels: {string: string}

  "Configuration specific to different kinds of components."
  component: ComponentKind @required

  "Assertions that can be run against the component to validate its behavior."
  tests: [TestDefinition]
}

union ComponentKind = WasmComponentConfiguration | CompositeComponentConfiguration

"A component made out of other components"
type CompositeComponentConfiguration @tagged("wick/component/composite@v1") {
  "Additional types to export and make available to the component."
  types: [TypeDefinition]

  "Components to import into the application's scope."
  import: [ComponentBinding]

  "A list of operations implemented by the Composite component."
  operations: [CompositeOperationDefinition]
}

"A component made out of other components"
type WasmComponentConfiguration @tagged("wick/component/wasmrs@v1") {
  "A reference to a local WebAssembly implementation"
  ref: LocationReference @rename("reference") @required

  "Additional types to export and make available to the component."
  types: [TypeDefinition]

  "A list of operations implemented by the WebAssembly module."
  operations: [OperationDefinition]
}

"An identifier bound to a resource."
type ResourceBinding {
  "The name of the binding."
  name: string @required
  "The resource to bind to."
  resource: ResourceDefinition @required
}

"An identifier bound to a component."
type ComponentBinding {
  "The name of the binding."
  name: string @required
  "The component to bind to."
  component: ComponentDefinition @required
}

"The possible types of components."
union ComponentDefinition = GrpcUrlComponent | ManifestComponent | ComponentReference | PostgresComponent

"A reference to a component in the application's scope."
type ComponentReference  @tagged("wick/component/reference@v1") {
  "The id of the component to reference."
  id: string  @required
}

"Host configuration options."
type HostConfig @default {
  "Whether or not to allow the :latest tag on remote artifacts."
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

"Per-component permissions configuration."
type Permissions @default {
  "A map of from internal directory to external directory that this component should be able to access."
  dirs: {string: string}
}

"A component hosted as an independent microservice."
type GrpcUrlComponent @tagged("wick/component/grpc@v1") {
  "The GRPC URL to connect to."
  url: string @required

  "Any configuration necessary for the component."
  config: any
}

"A native component that can be extracted and run as a microservice."
type ManifestComponent @tagged("wick/component/manifest@v1") {
  "The URL (and optional tag) or local file path to find the manifest."
  ref: LocationReference @rename("reference")  @required
  "Any configuration necessary for the component."
  config: any
}

"A definition for an single composite operation."
type CompositeOperationDefinition {
  "The name of the operation."
  name: string,

  "Types of the inputs to the operation."
  inputs: [Field],

  "Types of the outputs to the operation."
  outputs: [Field],

  "A list of components the schematic can use."
  components: [string],

  "A map of IDs to specific operation."
  instances: [InstanceBinding]

  "A list of connections from operation to operation."
  flow: [ConnectionDefinition],
}

"An operation name and its input and output signatures"
type OperationDefinition {
  "The name of the operation."
  name: string,

  "Types of the inputs to the operation."
  inputs: [Field],

  "Types of the outputs to the operation."
  outputs: [Field],
}

"Field definition. This is not technically an any type, it is a wick interface type field."
alias Field = any

"Type definition. This is not technically an any type, it is a wick interface type definition."
alias TypeDefinition = any

"An identifier bound to a component's operation."
type InstanceBinding {
  "The name of the binding."
  name: string @required
  "The operation to bind to."
  operation: ComponentOperationExpression @required
  "Data to associate with the reference, if any."
  config: any?,
}

"A connection between Operations and their ports. This can be specified in short-form syntax (where applicable)."
type ConnectionDefinition {
  "The upstream operation port."
  from: ConnectionTargetDefinition @required

  "The downstream operation port."
  to: ConnectionTargetDefinition @required
}

"A connection target e.g. a port on a reference. This can be specified in short-form syntax (where applicable)."
type ConnectionTargetDefinition {
  "The instance ID of the operation."
  instance: string @required,

  "The operation port."
  port: string @required,

  "The default value to provide on this connection in the event of an error."
  data: any?,
}

"A test case for a component."
type TestDefinition {
  "The name of the test."
  name: string @required

  "The operaton to test."
  operation: string @required

  "Inherent data to use for the test."
  inherent: InherentData?

  "The inputs to the test."
  input: [PacketData]

  "The expected outputs of the operation."
  output: [PacketData]
}

"Data inherent to transactions."
type InherentData {
  "An RNG seed."
  seed: u64?
  "A timestamp."
  timestamp: u64?
}

"Either a success packet or an error packet."
union PacketData @untagged = PayloadData | ErrorData

"A simplified representation of a Wick data packet & payload, used to write tests."
type PayloadData {
  "The name of the port to send the data to."
  name: string @required
  "Any flags set on the packet."
  flags: PacketFlags?
  "The data to send."
  data: any? @skip_env
}

type ErrorData {
  "The name of the port to send the data to."
  name: string @required
  "Any flags set on the packet."
  flags: PacketFlags?
  "The error message."
  message: string @required
}

"Flags set on a packet."
type PacketFlags {
  "When set, indicates the port should be considered closed."
  done: bool
  "When set, indicates the opening of a new substream context within the parent stream."
  open: bool
  "When set, indicates the closing of a substream context within the parent stream."
  close: bool
}


"A component made out of other components"
type PostgresComponent @tagged("wick/component/postgres@v1") {
  "The TcpPort reference to listen on for connections."
  resource: string

  "The username to use when connecting to the postgres database."
  user: string

  "The password to use when connecting to the postgres database."
  password: string

  "The database to connect to."
  database: string

  "Whether or not to use TLS."
  tls: bool

  "A list of operations to expose on this component."
  operations: [PostgresOperationDefinition]
}

type PostgresOperationDefinition {
  "The name of the operation."
  name: string

  "Types of the inputs to the operation."
  inputs: [Field],

  "Types of the outputs to the operation."
  outputs: [Field],

  "The query to execute."
  query: string

  "The arguments to the query, defined as a list of input names."
  arguments: [string]
}

