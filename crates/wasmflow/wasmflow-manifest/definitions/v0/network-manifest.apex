
"A Wasmflow network definition."
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

  "Data or configuration used to initialize the collection."
  data: struct,
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
  data: struct,
}

"Kind of collection."
enum CollectionKind {
  "Native collections included at compile-time in a Wasmflow host."
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
  Par = 5,
}
