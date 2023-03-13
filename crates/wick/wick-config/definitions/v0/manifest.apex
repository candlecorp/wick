namespace "wick-config::v0."

"A manifest defines the starting state of a Wick host and network."
type HostManifest {
  "The configuration manifest format."
  format: u32 @required,

  "The version of the configuration."
  version: string,

  "Additional host configuration."
  host: HostConfig,

  "The configuration for a Wick network."
  network: NetworkManifest,

  "The default schematic to execute if none is provided."
  default_schematic: string?,
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

  "Configuration for the HTTP 1 server (development only)."
  http: HttpConfig?
}

"Configuration for HTTP/S servers."
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


"A Wick network definition."
type NetworkManifest {
  "The unique identifier for this Network."
  name: string?,

  "The labels that apply to this network."
  labels: {string: string},

  "The collection to use as the entrypoint when running as a standalone process."
  triggers: EntrypointDefinition?,

  "The links between capabilities and components."
  schematics: [SchematicManifest],

  "A list of component collections."
  collections: [CollectionDefinition],
}

"A collection definition for the main entrypoint."
type EntrypointDefinition {
  "The reference/location of the collection."
  reference: string,

  "The component to use as the entrypoint."
  component: string,

  "Data or configuration used to initialize the collection."
  data: any,
}

"A collection definition."
type CollectionDefinition {
  "The local namespace for the collection."
  namespace: string,

  "The kind/type of the collection."
  kind: CollectionKind,

  "The reference/location of the collection."
  reference: string,

  "Data or configuration used to initialize the collection."
  data: any,
}

"Kind of collection."
enum CollectionKind {
  "Native collections included at compile-time in a Wick host."
  Native = 0,
  "The URL for a separately managed GRPC endpoint."
  GrpcUrl = 1,
  "A WaPC WebAssembly collection."
  WaPC = 2,
  "A collection accessible via a connected mesh."
  Mesh = 3,
  "A local or remote Network definition."
  Network = 4,
  "A GRPC collection binary."
  GrpcTar = 5,
}


"A definition for an individual Wick schematic."
type SchematicManifest {
  "Schematic name."
  name: string @required,

  "A list of collections the schematic uses."
  collections: [string],

  "A map from component reference to its target."
  instances: {string: ComponentDefinition},

  "A list of connections from component to component."
  connections: [ConnectionDefinition],

  "A map of constraints and values that limit where this schematic can run."
  constraints: {string: string},
}


"A single component definition."
type ComponentDefinition {
  "The ID of the component (i.e. the alias, key, or namespace)."
  id: string @required,

  "Data to associate with the reference."
  data: any?,
}

"A connection between components. This can be specified in short-form syntax (where applicable)."
type ConnectionDefinition {
  "The originating component from upstream."
  from: ConnectionTargetDefinition,

  "The destination component (downstream)."
  to: ConnectionTargetDefinition,

  "The default value to provide in the event of an upstream Error or Exception."
  default: string?
}

"A connection target e.g. a port on a reference. This can be specified in short-form syntax (where applicable)."
type ConnectionTargetDefinition {
  "The instance name of the referenced component."
  instance: string @required,
  "The component's port."
  port: string @required,
  "Data to associate with a connection."
  data: any?,
}
