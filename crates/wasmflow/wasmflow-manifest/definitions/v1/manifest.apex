namespace "wasmflow-manifest::v1"

"A manifest defines the starting state of a Wasmflow host and network."
type WasmflowManifest {
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

  "The collection to use as the entrypoint when running as a standalone process."
  unstable_triggers: EntrypointDefinition?,

  "A map of namespace to external component collection."
  collections: {string: CollectionDefinition},

  "A map of flow-component names to their definitions."
  components: {string: FlowDefinition},
}

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

"A collection definition for the main entrypoint."
type EntrypointDefinition {
  "The reference/location of the collection."
  reference: string,

  "Data or configuration used to initialize the collection."
  config: struct,
}

"A collection definition."
type CollectionDefinition {
  "The kind/type of the collection."
  kind: CollectionKind,

  "The reference/location of the collection."
  reference: string,

  "Data or configuration used to initialize the collection."
  config: struct,
}

"Kind of collection."
enum CollectionKind {
  "Native collections included at compile-time in a Wasmflow host."
  Native = 0,
  "The URL to an external collection via the GRPC interface."
  GrpcUrl = 1,
  "A WebAssembly collection."
  WASM = 2,
  "A collection accessible in a Wasmflow mesh."
  Mesh = 3,
  "A local or remote Wasmflow manifest."
  Network = 4,
  "A native binary archive that contains a GRPC-capable collection."
  Par = 5,
}

"A definition for an single flow."
type FlowDefinition {
  "A list of collections the schematic can use."
  collections: [string],

  "A map from component reference to its target."
  instances: {string: ComponentDefinition},

  "A list of connections from component to component."
  flow: [ConnectionDefinition],
}

"A single component definition."
type ComponentDefinition {
  "The ID of the component (i.e. the alias, key, or namespace)."
  id: string @required,

  "Data to associate with the reference."
  config: struct?,
}

"A connection between components. This can be specified in short-form syntax (where applicable). See <a href='https://wasmflow.com/docs/configuration/short-form-syntax/'>wasmflow.com</a> for more information."
type ConnectionDefinition {
  "The originating component from upstream."
  from: ConnectionTargetDefinition,

  "The destination component (downstream)."
  to: ConnectionTargetDefinition,

  "The default value to provide in the event of an upstream Error or Exception."
  default: string?
}

"A connection target e.g. a port on a reference. This can be specified in short-form syntax (where applicable).  See <a href='https://wasmflow.com/docs/configuration/short-form-syntax/'>wasmflow.com</a> for more information."
type ConnectionTargetDefinition {
  "The instance name of the referenced component."
  instance: string @required,

  "The component's port."
  port: string @required,

  "The default value to provide on this connection in the event of an error."
  data: struct?,
}
