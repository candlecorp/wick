use std::collections::{HashMap, HashSet};

use openapiv3::{
  ArrayType,
  Components,
  Contact,
  Info,
  IntegerType,
  License,
  NumberType,
  ObjectType,
  OpenAPI,
  Operation,
  Parameter,
  ParameterData,
  ParameterSchemaOrContent,
  PathItem,
  Paths,
  ReferenceOr,
  Responses,
  Schema,
  SchemaData,
  StringType,
  VariantOrUnknownOrEmpty,
};
use wick_config::config::{AppConfiguration, HttpMethod, WickRouter};
use wick_interface_types::{EnumDefinition, Field, StructDefinition, Type, TypeDefinition, UnionDefinition};

use super::error::RestError;
use super::RestRoute;

// Generates an OpenAPI spec from a RestRouterConfig
pub(crate) fn generate_openapi(
  app_config: &AppConfiguration,
  config: &wick_config::config::RestRouterConfig,
  routes: &[RestRoute],
) -> Result<OpenAPI, RestError> {
  let mut openapi = OpenAPI::default();

  let info = config.info().ok_or(RestError::MissingConfig("info".to_owned()))?;

  openapi.openapi = "3.0.0".to_owned();

  openapi.info = Info {
    title: info.title().cloned().unwrap_or_else(|| "Untitled API".to_owned()),
    description: info.description().cloned(),
    terms_of_service: info.tos().cloned(),
    contact: convert_contact(info.contact()),
    license: convert_license(info.license()),
    version: info.version().to_owned(),
    extensions: Default::default(),
  };
  openapi.servers = vec![openapiv3::Server {
    url: config.path().to_owned(),
    description: info.description().cloned(),
    variables: Default::default(),
    extensions: Default::default(),
  }];
  let mut named_types = HashSet::new();

  let paths: Vec<(_, _)> = routes
    .iter()
    .map(|route| {
      let path_item = route_to_path_item(route, &mut named_types);
      let path = route
        .route
        .path_parts
        .iter()
        .map(|p| match p {
          super::route::PathPart::Literal(part) => part.clone(),
          super::route::PathPart::Param(param) => format!("{{{}}}", param.name()),
        })
        .collect::<Vec<_>>()
        .join("/");
      (format!("/{}", path), ReferenceOr::Item(path_item))
    })
    .collect();
  openapi.paths = Paths {
    paths: Default::default(),
    extensions: Default::default(),
  };
  openapi.paths.paths.extend(paths.into_iter());

  let (mut schemas, mut named_types, mut new_named_types) =
    resolve_named_types(app_config, HashMap::new(), named_types)?;
  while !new_named_types.is_empty() {
    (schemas, named_types, new_named_types) = resolve_named_types(app_config, schemas, named_types)?;
  }

  if openapi.components.is_none() {
    openapi.components = Some(Components::default());
  };
  let mut components = Components::default();
  components.schemas.extend(schemas.into_iter());

  openapi.components = Some(components);
  Ok(openapi)
}

type Seen = HashSet<String>;
type SchemaMap = HashMap<String, ReferenceOr<Schema>>;

fn resolve_named_types(
  app_config: &AppConfiguration,
  mut schemas: HashMap<String, ReferenceOr<Schema>>,
  mut named: HashSet<String>,
) -> Result<(SchemaMap, Seen, Seen), RestError> {
  let mut new_named = HashSet::new();

  for name in &named {
    if !schemas.contains_key(name) {
      let ty = app_config
        .resolve_type(name)
        .ok_or_else(|| RestError::TypeNotFound(name.clone()))?;
      let schema = typedef_to_schema(&ty, &mut new_named);
      schemas.insert(name.clone(), ReferenceOr::Item(schema));
    }
  }
  for new_name in &new_named {
    named.insert(new_name.clone());
  }
  Ok((schemas, named, new_named))
}

fn route_to_path_item(route: &RestRoute, named: &mut HashSet<String>) -> PathItem {
  let mut path_item = PathItem {
    summary: route.config.summary().cloned(),
    description: route.config.description().cloned(),
    ..Default::default()
  };

  for field in route
    .route
    .path_parts
    .iter()
    .filter(|p| p.is_param())
    .filter_map(|p| p.param())
  {
    path_item.parameters.push(ReferenceOr::Item(Parameter::Path {
      parameter_data: field_to_parameter_data(field, named),
      style: openapiv3::PathStyle::Simple,
    }));
  }
  for field in route.route.query_params.iter() {
    path_item.parameters.push(ReferenceOr::Item(Parameter::Query {
      parameter_data: field_to_parameter_data(field, named),
      style: openapiv3::QueryStyle::DeepObject,
      allow_reserved: Default::default(),
      allow_empty_value: Some(matches!(field.ty(), Type::Optional { .. })),
    }));
  }
  let oapi_operation = Operation {
    tags: Default::default(),
    operation_id: route.config.id().cloned(),
    summary: None,
    description: None,
    external_docs: Default::default(),
    parameters: Default::default(),
    request_body: Default::default(),
    responses: Responses::default(),
    deprecated: Default::default(),
    security: Default::default(),
    servers: Default::default(),
    extensions: Default::default(),
  };
  if route.config.methods().is_empty() || route.config.methods().contains(&HttpMethod::Get) {
    path_item.get = Some(oapi_operation.clone());
  }
  if route.config.methods().contains(&HttpMethod::Post) {
    path_item.post = Some(oapi_operation.clone());
  }
  if route.config.methods().contains(&HttpMethod::Put) {
    path_item.put = Some(oapi_operation.clone());
  }
  if route.config.methods().contains(&HttpMethod::Delete) {
    path_item.delete = Some(oapi_operation);
  }
  path_item
}

fn convert_contact(contact: Option<&wick_config::config::Contact>) -> Option<Contact> {
  contact.map(|c| Contact {
    name: c.name().cloned(),
    url: c.url().cloned(),
    email: c.email().cloned(),
    extensions: Default::default(),
  })
}

fn convert_license(license: Option<&wick_config::config::License>) -> Option<License> {
  license.map(|l| License {
    name: l.name().to_owned(),
    url: l.url().cloned(),
    extensions: Default::default(),
  })
}

fn field_to_parameter_data(field: &Field, named: &mut HashSet<String>) -> ParameterData {
  ParameterData {
    name: field.name().to_owned(),
    description: field.description().map(|s| s.to_owned()),
    required: field.required(),
    format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(field_to_schema(field, named))),
    deprecated: Default::default(),
    example: Default::default(),
    examples: Default::default(),
    explode: Default::default(),
    extensions: Default::default(),
  }
}

fn field_to_schema(field: &Field, named: &mut HashSet<String>) -> Schema {
  Schema {
    schema_data: SchemaData {
      description: field.description().map(|s| s.to_owned()),
      nullable: matches!(field.ty(), Type::Optional { .. }),
      default: field.default().cloned(),
      ..Default::default()
    },
    schema_kind: wick_type_to_schemakind(field.ty(), named),
  }
}

fn typedef_to_schema(ty: &TypeDefinition, named: &mut HashSet<String>) -> Schema {
  match ty {
    TypeDefinition::Struct(v) => struct_to_schema(v, named),
    TypeDefinition::Enum(v) => enum_to_schema(v, named),
    TypeDefinition::Union(v) => union_to_schema(v, named),
  }
}

fn struct_to_schema(ty: &StructDefinition, named: &mut HashSet<String>) -> Schema {
  let mut obj = ObjectType::default();
  for field in &ty.fields {
    obj.properties.insert(
      field.name().to_owned(),
      ReferenceOr::Item(Box::new(field_to_schema(field, named))),
    );
    if field.required() {
      obj.required.push(field.name().to_owned());
    }
  }
  Schema {
    schema_data: SchemaData {
      description: ty.description.clone(),
      nullable: false,
      ..Default::default()
    },
    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(obj)),
  }
}

fn enum_to_schema(ty: &EnumDefinition, _named: &mut HashSet<String>) -> Schema {
  let mut variants = Vec::new();
  for variant in &ty.variants {
    variants.push(Some(variant.name.clone()));
  }
  Schema {
    schema_data: SchemaData {
      description: ty.description.clone(),
      nullable: false,
      ..Default::default()
    },
    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
      enumeration: variants,
      ..Default::default()
    })),
  }
}

fn union_to_schema(ty: &UnionDefinition, named: &mut HashSet<String>) -> Schema {
  let mut variants = Vec::new();
  for variant in &ty.types {
    variants.push(ReferenceOr::Item(type_to_schema(variant, named)));
  }
  Schema {
    schema_data: SchemaData {
      description: ty.description.clone(),
      nullable: false,
      ..Default::default()
    },
    schema_kind: openapiv3::SchemaKind::OneOf { one_of: variants },
  }
}

fn type_to_schema(ty: &Type, named: &mut HashSet<String>) -> Schema {
  Schema {
    schema_data: SchemaData {
      nullable: matches!(ty, Type::Optional { .. }),
      ..Default::default()
    },
    schema_kind: wick_type_to_schemakind(ty, named),
  }
}
macro_rules! int_type {
  ($ty:ty, $format:expr) => {
    openapiv3::SchemaKind::Type(openapiv3::Type::Integer(IntegerType {
      format: VariantOrUnknownOrEmpty::Item($format),
      multiple_of: Default::default(),
      exclusive_minimum: Default::default(),
      exclusive_maximum: Default::default(),
      minimum: Some(<$ty>::MIN as _),
      maximum: Some(<$ty>::MAX as _),
      enumeration: Default::default(),
    }))
  };
}

macro_rules! float_type {
  ($ty:ty, $format:expr) => {
    openapiv3::SchemaKind::Type(openapiv3::Type::Number(NumberType {
      format: VariantOrUnknownOrEmpty::Item($format),
      multiple_of: Default::default(),
      exclusive_minimum: Default::default(),
      exclusive_maximum: Default::default(),
      minimum: Some(<$ty>::MIN as _),
      maximum: Some(<$ty>::MAX as _),
      enumeration: Default::default(),
    }))
  };
}

fn wick_type_to_schemakind(ty: &Type, named: &mut HashSet<String>) -> openapiv3::SchemaKind {
  #[allow(trivial_numeric_casts)]
  match ty {
    Type::I8 => int_type!(i8, openapiv3::IntegerFormat::Int32),
    Type::I16 => int_type!(i16, openapiv3::IntegerFormat::Int32),
    Type::I32 => int_type!(i32, openapiv3::IntegerFormat::Int32),
    Type::I64 => int_type!(i64, openapiv3::IntegerFormat::Int64),
    Type::U8 => int_type!(u8, openapiv3::IntegerFormat::Int32),
    Type::U16 => int_type!(u16, openapiv3::IntegerFormat::Int32),
    Type::U32 => int_type!(u32, openapiv3::IntegerFormat::Int32),
    Type::U64 => int_type!(u64, openapiv3::IntegerFormat::Int64),
    Type::F32 => float_type!(f32, openapiv3::NumberFormat::Float),
    Type::F64 => float_type!(f64, openapiv3::NumberFormat::Double),
    Type::Bool => openapiv3::SchemaKind::Type(openapiv3::Type::Boolean {}),
    Type::String => openapiv3::SchemaKind::Type(openapiv3::Type::String(Default::default())),
    Type::Datetime => openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
      format: VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::DateTime),
      ..Default::default()
    })),
    Type::Bytes => openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
      format: VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Binary),
      ..Default::default()
    })),
    Type::Named(name) => {
      named.insert(name.clone());
      openapiv3::SchemaKind::OneOf {
        one_of: vec![ReferenceOr::Reference {
          reference: name.clone(),
        }],
      }
    }
    Type::List { ty } => openapiv3::SchemaKind::Type(openapiv3::Type::Array(ArrayType {
      items: Some(ReferenceOr::Item(Box::new(type_to_schema(ty, named)))),
      min_items: Default::default(),
      max_items: Default::default(),
      unique_items: Default::default(),
    })),
    // The optionality needs to be handled one level above, so just convert the inner type.
    Type::Optional { ty } => wick_type_to_schemakind(ty, named),
    Type::Map { value, .. } => openapiv3::SchemaKind::Type(openapiv3::Type::Object(ObjectType {
      additional_properties: Some(openapiv3::AdditionalProperties::Schema(Box::new(ReferenceOr::Item(
        type_to_schema(value, named),
      )))),
      properties: Default::default(),
      required: Default::default(),
      min_properties: Default::default(),
      max_properties: Default::default(),
    })),
    Type::Object => openapiv3::SchemaKind::Type(openapiv3::Type::Object(ObjectType {
      additional_properties: Some(openapiv3::AdditionalProperties::Any(true)),
      properties: Default::default(),
      required: Default::default(),
      min_properties: Default::default(),
      max_properties: Default::default(),
    })),
    Type::AnonymousStruct(_) => unimplemented!("Anonymous structs are not supported in OpenAPI config"),
    #[allow(deprecated)]
    Type::Link { .. } => unreachable!(),
  }
}
