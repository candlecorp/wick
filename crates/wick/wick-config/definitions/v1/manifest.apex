namespace "wick-config::v1"

"Configuration for Wick applications and components."
union WickConfig = AppConfiguration | ComponentConfiguration

"The Application configuration defines a standalone Wick application."
type AppConfiguration @default {
  "The configuration version."
  format: u32 @required,

  "Associated metadata for this component."
  metadata: AppMetadata?,

  "The application's name."
  name: string,

  "Components to import into the application's scope."
  import: [ComponentBinding],

  "Resources that the application can access."
  resources: [ResourceBinding],

  "Configured triggers that drive the application's behavior."
  triggers: [TriggerDefinition],
}

"Metadata for the application."
type AppMetadata @default {
  "The version of the application."
  version: string,
}

"The possible types of resources."
union TriggerDefinition = CliTrigger | HttpTrigger

"A trigger called with a CLI context."
type CliTrigger @internal_tag("kind", "wick/trigger/cli@v1") {
  "The operation that will act as the main entrypoint for this trigger."
  operation: ComponentOperationExpression @required
  "The component that provides additional logic."
  app: ComponentDefinition? @shortform
}

"A reference to an operation with an explicit component definition."
type ComponentOperationExpression {
  "The component that exports the operation."
  component: ComponentDefinition @required @shortform
  "The operation to call."
  operation: string @required
}

"An HTTP server that delegates to HTTP routers upon requests."
type HttpTrigger @internal_tag("kind", "wick/trigger/http@v1") {
  "The TcpPort reference to listen on for connections."
  resource: string

  "The HttpRouters that should handle incoming requests"
  routers: [HttpRouter]
}

union HttpRouter = RawRouter | RestRouter

type RestRouter @internal_tag("kind", "wick/router/rest@v1") {
  "The path to start serving this router from."
  path: string @required

  "The component to expose as a Rest API."
  component: ComponentDefinition @required
}

"A RawHttpRouter delegates raw requests and bodies to operations based on the request path."
type RawRouter @internal_tag("kind", "wick/router/raw@v1") {
  "The path to start serving this router from."
  path: string @required

  "The operation that handles HTTP requests."
  operation: ComponentOperationExpression @required
}

"The possible types of resources."
union ResourceDefinition = TcpPort | UdpPort

"A WebAssembly component."
type TcpPort @internal_tag("kind", "wick/resource/tcpport@v1") {
  "The port to bind to."
  port: u16
  "The address to bind to."
  address: string
}

"A WebAssembly component."
type UdpPort @internal_tag("kind", "wick/resource/udpport@v1") {
  "The port to bind to."
  port: u16
  "The address to bind to."
  address: string
}

"A manifest defines the starting state of a Wick host and network."
type ComponentConfiguration @default {
  "The name of this component."
  name: string?,

  "The component manifest format version"
  format: u32 @required,

  "Associated metadata for this component."
  metadata: ComponentMetadata?,

  "Configuration for the host when this manifest is run directly."
  host: HostConfig,

  "The labels and values that apply to this manifest."
  labels: {string: string},

  "Additional types to export and make available to the component."
  types: [TypeDefinition],

  "Components to import into the application's scope."
  import: [ComponentBinding],

  "A map of operation names to their definitions."
  operations: [OperationDefinition],
}

"Metadata for the component."
type ComponentMetadata @default {
  "The version of the component."
  version: string,
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
union ComponentDefinition = WasmComponent | GrpcUrlComponent | ManifestComponent | ComponentReference

"A reference to a component in the application's scope."
type ComponentReference  @internal_tag("kind", "Reference") {
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
  pem: string?,

  "Path to key file for TLS."
  key: string?,

  "Path to CA file."
  ca: string?,
}

"A WebAssembly component."
type WasmComponent @internal_tag("kind", "Wasm") {
  "The URL (and optional tag) or local file path to find the .wasm module."
  ref: string @rename("reference") @required

  "Permissions to give this component"
  permissions: Permissions

  "Per-component configuration."
  config: any
}

"Per-component permissions configuration."
type Permissions @default {
  "A map of from internal directory to external directory that this component should be able to access."
  dirs: {string: string}
}

"A component hosted as an independent microservice."
type GrpcUrlComponent @internal_tag("kind", "GrpcUrl") {
  "The GRPC URL to connect to."
  url: string @required

  "Any configuration necessary for the component."
  config: any
}

"A native component that can be extracted and run as a microservice."
type ManifestComponent @internal_tag("kind", "Manifest") {
  "The URL (and optional tag) or local file path to find the manifest."
  ref: string @rename("reference")  @required
  "Any configuration necessary for the component."
  config: any
}

"A definition for an single flow."
type OperationDefinition {
  "The name of the operation."
  name: string,

  "Types of the inputs to the operation."
  inputs: [Field],

  "Types of the outputs to the operation."
  outputs: [Field],

  "A list of components the schematic can use."
  components: [string],

  "A map of IDs to specific operation."
  instances: {string: InstanceDefinition},

  "A list of connections from operation to operation."
  flow: [ConnectionDefinition],
}

"Field definition. This is not technically an any type, it is a wick interface type field."
alias Field = any

"Type definition. This is not technically an any type, it is a wick interface type definition."
alias TypeDefinition = any

"The ID and configuration for an operation."
type InstanceDefinition {
  "The ID to assign to this instance of the operation."
  id: string @required,

  "Data to associate with the reference."
  config: any?,
}

"A connection between Operations and their ports. This can be specified in short-form syntax (where applicable)."
type ConnectionDefinition {
  "The upstream operation port."
  from: ConnectionTargetDefinition @required

  "The downstream operation port."
  to: ConnectionTargetDefinition @required

  "The default value to provide in the event of an upstream Error or Exception."
  default: string?
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
