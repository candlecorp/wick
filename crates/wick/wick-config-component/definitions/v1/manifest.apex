namespace "wick-config-component::v1"

"A manifest defines the starting state of a Wick host and network."
type ComponentConfiguration {
  "The manifest version."
  version: u8,

  "Configuration for the host when this manifest is run directly."
  host: HostConfig,

  "The default flow to execute if none is provided."
  default_flow: string?,

  "The unique identifier for this manifest."
  name: string?,

  "The labels and values that apply to this manifest."
  labels: {string: string},

  "A map of namespace to external components."
  external: {string: ComponentDefinition},

  "A map of operation names to their definitions."
  operations: {string: FlowDefinition},
}

"The possible types of components."
union ComponentDefinition = WasmComponent | GrpcUrlComponent | GrpcTarComponent | MeshComponent | ManifestComponent

"Host configuration options."
type HostConfig {
  "Whether or not to allow the :latest tag on remote artifacts."
  allow_latest: bool,

  "A list of registries to connect to insecurely (over HTTP vs HTTPS)."
  insecure_registries: [string],

  "The timeout for network requests (in ms)."
  timeout: u64 = 5000,

  "The ID for this host, used to identify the host over the mesh."
  id: string?,

  "The schematics to expose via RPC or the mesh, if any."
  expose: [string]

  "The mesh configuration."
  mesh: MeshConfig?

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

"Configuration used to connect to the mesh."
type MeshConfig {
  "Enable/disable the mesh connection."
  enabled: bool,

  "The address of the NATS server."
  address: string,

  "The path to the NATS credsfile."
  creds_path: string?,

  "The NATS token."
  token: string?,
}

"A WebAssembly component."
type WasmComponent @internal_tag("kind", "Wasm") {
  "The URL (and optional tag) or local file path to find the .wasm module."
  reference: string,

  "Permissions to give this component"
  permissions: Permissions

  "Per-component configuration."
  config: any
}

"Per-component permissions configuration."
type Permissions {
  "A map of from internal directory to external directory that this component should be able to access."
  dirs: {string: string}
}

"A component hosted as an independent microservice."
type GrpcUrlComponent  @internal_tag("kind", "GrpcUrl") {
  "The GRPC URL to connect to."
  url: string,
  "Any configuration necessary for the component."
  config: any
}

"A component hosted somewhere on a connected mesh."
type MeshComponent  @internal_tag("kind", "Mesh") {
  "The ID of the component."
  id: string,
  "Any configuration necessary for the component."
  config: any
}

"A native component that can be extracted and run as a microservice."
type GrpcTarComponent  @internal_tag("kind", "GrpcTar") {
  "The URL (and optional tag) or local file path to find the archive."
  reference: string,
  "Any configuration necessary for the component."
  config: any
}

"A native component that can be extracted and run as a microservice."
type ManifestComponent  @internal_tag("kind", "Manifest") {
  "The URL (and optional tag) or local file path to find the manifest."
  reference: string,
  "Any configuration necessary for the component."
  config: any
}

"A definition for an single flow."
type FlowDefinition {
  "A list of components the schematic can use."
  components: [string],

  "A map of IDs to specific operation."
  instances: {string: InstanceDefinition},

  "A list of connections from operation to operation."
  flow: [ConnectionDefinition],
}

"The ID and configuration for an operation."
type InstanceDefinition {
  "The ID to assign to this instance of the operation."
  id: string @required,

  "Data to associate with the reference."
  config: any?,
}

"A connection between Operations and their ports. This can be specified in short-form syntax (where applicable). See <a href='https://wasmflow.com/docs/configuration/short-form-syntax/'>wasmflow.com</a> for more information."
type ConnectionDefinition {
  "The upstream operation port."
  from: ConnectionTargetDefinition,

  "The downstream operation port."
  to: ConnectionTargetDefinition,

  "The default value to provide in the event of an upstream Error or Exception."
  default: string?
}

"A connection target e.g. a port on a reference. This can be specified in short-form syntax (where applicable).  See <a href='https://wasmflow.com/docs/configuration/short-form-syntax/'>wasmflow.com</a> for more information."
type ConnectionTargetDefinition {
  "The instance ID of the operation."
  instance: string @required,

  "The operation port."
  port: string @required,

  "The default value to provide on this connection in the event of an error."
  data: any?,
}
