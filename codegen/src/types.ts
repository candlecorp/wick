export type WIDL_TYPE =
  | 'i8'
  | 'u8'
  | 'i16'
  | 'u16'
  | 'i32'
  | 'u32'
  | 'i64'
  | 'u64'
  | 'f32'
  | 'f64'
  | 'bool'
  | 'string'
  | 'datetime'
  | 'bytes'
  | 'raw'
  | 'value';

export const WIDL_TYPE_LIST = [
  'i8',
  'u8',
  'i16',
  'u16',
  'i32',
  'u32',
  'i64',
  'u64',
  'f32',
  'f64',
  'bool',
  'string',
  'datetime',
  'bytes',
  'raw',
  'value',
];

export type FieldMap = Record<string, TypeSignature>;

export type TypeSignature = SimpleType | StructType | RefType | ListType | OptionalType | MapType | LinkType;

export interface CollectionSignature {
  name: string;
  format: number;
  version: string;
  wellknown?: WellKnownSchema[];
  types: Record<string, RootType>;
  components: Record<string, ComponentSignature>;
  config?: Record<string, StructSignature>;
}

export interface WellKnownSchema {
  capabilities: string[];
  url: string;
  schema: CollectionSignature;
}

export interface ComponentSignature {
  name: string;
  inputs: FieldMap;
  outputs: FieldMap;
}

export type RootType = StructSignature | EnumSignature;

export interface EnumSignature {
  type: 'enum';
  name: string;
  values: EnumVariant[];
}

export interface EnumVariant {
  name: string;
  index: number;
}

export interface StructSignature {
  type: 'struct';
  name: string;
  fields: Record<string, TypeSignature>;
}

export interface SimpleType {
  type: WIDL_TYPE;
}

export interface StructType {
  type: 'struct';
}

export interface RefType {
  type: 'ref';
  ref: string;
}

export interface MapType {
  type: 'map';
  key: TypeSignature;
  value: TypeSignature;
}

export interface ListType {
  type: 'list';
  element: TypeSignature;
}

export interface OptionalType {
  type: 'optional';
  option: TypeSignature;
}

export interface LinkType {
  type: 'link';
  collection?: string;
}

export function isWidlType(type: string): type is WIDL_TYPE {
  return WIDL_TYPE_LIST.includes(type);
}
