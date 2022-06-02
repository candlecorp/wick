namespace "wasmflow::interface"

"An enum representing the types of components that can be hosted."
enum HostedType {
  "A provider."
  Provider = 0,
}

alias FieldMap = {string: TypeSignature}

union Help = ProviderSignature | ComponentSignature

type ProviderSignature {
  "Name of the provider."
  name: string?,
  "Schema format version."
  format: u32,
  "Version of the schema."
  version: string,
  "A map of type signatures referenced elsewhere."
  wellknown: [WellKnownSchema],
  "A map of type signatures referenced elsewhere."
  types: {string: TypeSignature},
  "A list of ComponentSignatures the provider hosts."
  components: {string: ComponentSignature},
  "The component's configuration for this implementation."
  config: {string: TypeSignature},
}

"The signature of a Wasmflow component, including its input and output types."
type ComponentSignature {
  "The name of the component."
  name: string,
  "The component's inputs."
  inputs: {string: TypeSignature},
  "The component's outputs."
  outputs: {string: TypeSignature},
}


"An entry from a well-known schema"
type WellKnownSchema {
  "The capability the schema provides."
  capabilities: [string],
  "The location where you can find and validate the schema."
  url: string,
  "The schema itself."
  schema: ProviderSignature,
}

"Signatures of enum type definitions."
type EnumSignature {
  "The name of the enum."
  name: string,
  "The variants in the enum."
  values: [EnumVariant],
}

"An enum variant definition"
type EnumVariant {
  "The name of the variant."
  name: string,
  "The index of the variant."
  index: u32,
}

"Signatures of struct-like type definitions."
type StructSignature {
  "The name of the struct."
  name: string,
  "The fields in this struct."
  fields: {string: TypeSignature},
}

"Enum of valid types."
enum TypeSignature {
  "I8 type."
  I8 = 0,
  "I16 type."
  I16 = 1,
  "I32 type."
  I32 = 2,
  "I64 type."
  I64 = 3,
  "u8 type."
  U8 = 4,
  "u16 type."
  U16 = 5,
  "u32 type."
  U32 = 6,
  "u64 type."
  U64 = 7,
  "f32 type."
  F32 = 8,
  "f64 type."
  F64 = 9,
  "Boolean type."
  Bool = 10,
  "string type."
  String = 11,
  "Date type."
  Datetime = 12,
  "Raw bytes."
  Bytes = 13,
  "Any valid value."
  Value = 14,
  "An internal type."
  Internal = 15,
  "A reference to another type."
  Ref = 16,
  "A list type"
  List = 17,
  "A type representing an optional value."
  Optional = 18,
  "A HashMap-like type."
  Map = 19,
  "A type representing a ProviderLink."
  Link = 20,
  "A JSON-like key/value map."
  Struct = 21,
}

"Internal types for use within the Wasmflow runtime"
enum InternalType {
  "Represents a complete set of component inputs"
  ComponentInput = 0,
}
