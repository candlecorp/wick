use serde_json::Value;
use wick_interface_types::{Field, FieldValue, Type};

use crate::triggers::http::HttpError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathPart {
  Literal(String),
  Param(Field),
}

impl PathPart {
  pub(crate) fn is_param(&self) -> bool {
    matches!(self, Self::Param(_))
  }

  pub(crate) fn param(&self) -> Option<&Field> {
    match self {
      Self::Param(field) => Some(field),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Route {
  pub(crate) path_parts: Vec<PathPart>,
  pub(crate) query_params: Vec<Field>,
}

pub(super) type MatchedValues = (Vec<FieldValue>, Vec<FieldValue>);

impl Route {
  pub(super) fn parse(path: &str) -> Result<Self, wick_interface_types::ParserError> {
    let (path, query) = path.find('?').map_or((path, ""), |idx| {
      let (path, query) = path.split_at(idx);
      (path, query.trim_start_matches('?'))
    });
    let mut path_parts = Vec::new();
    let mut query_params = Vec::new();

    let path = path.trim_start_matches('/');

    for part in path.split('/') {
      if part.starts_with('{') && part.ends_with('}') {
        let name = part[1..part.len() - 1].to_owned();
        let (name, type_) = if let Some(idx) = name.find(':') {
          let (name, type_) = name.split_at(idx);
          let type_ = type_.trim_start_matches(':');
          let ty = wick_interface_types::parse(type_)?;
          (name.to_owned(), ty)
        } else {
          (name, Type::Object)
        };
        let field = Field::new(name, type_);
        path_parts.push(PathPart::Param(field));
      } else {
        path_parts.push(PathPart::Literal(part.to_owned()));
      }
    }

    let query_parts: Vec<&str> = if query.contains('&') {
      query.split('&').collect()
    } else if !query.is_empty() {
      vec![query]
    } else {
      vec![]
    };

    for part in query_parts {
      let (name, type_) = if let Some(idx) = part.find(':') {
        let (name, type_) = part.split_at(idx);
        let type_ = type_.trim_start_matches(':');
        let ty = wick_interface_types::parse(type_)?;
        (name.to_owned(), ty)
      } else {
        (part.to_owned(), Type::Object)
      };
      query_params.push(Field::new(name, type_));
    }

    trace!(path_parts=?path_parts,query_parts=?query_params, "processed url path and query params");

    Ok(Self {
      path_parts,
      query_params,
    })
  }

  pub(super) fn compare(&self, path: &str, query_string: Option<&str>) -> Result<Option<MatchedValues>, HttpError> {
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();

    let path = path.trim_start_matches('/');

    let mut path_parts = path.split('/').peekable();
    for part in &self.path_parts {
      match part {
        PathPart::Literal(literal) => {
          if let Some(part) = path_parts.next() {
            if part != *literal {
              return Ok(None);
            }
          } else {
            return Ok(None);
          }
        }
        PathPart::Param(field) => {
          if let Some(part) = path_parts.next() {
            let Ok(value) = field.ty.coerce_str(part) else {
              warn!("Failed to coerce {:?} to {:?}", part, field.ty);
              return Err(HttpError::InvalidParameter(field.name.clone()));
            };
            path_params.push(field.clone().with_value(value));
          } else {
            return Ok(None);
          }
        }
      }
    }

    if let Some(query_string) = query_string {
      if query_string.trim().is_empty() && !self.query_params.is_empty() {
        return Err(HttpError::MissingQueryParameters(
          self.query_params.iter().map(|p| p.name.clone()).collect(),
        ));
      }
      for param in &self.query_params {
        let params = query_string.split('&').peekable();
        for query_param in params {
          let (name, value) = query_param.find('=').map_or_else(
            || (query_param.to_owned(), ""),
            |idx| {
              let (name, value) = query_param.split_at(idx);
              let value = value.trim_start_matches('=');
              (name.to_owned(), value)
            },
          );
          if name != param.name {
            continue;
          }

          if let Type::List { ty } = &param.ty {
            let Ok(value) = ty.coerce_str(value) else {
              warn!("Failed to coerce {} to {} for query param {}", value, param.ty,name);
              return Err(HttpError::InvalidParameter(param.name.clone()));
            };

            if let Some(field) = query_params
              .iter_mut()
              .find(|p: &&mut FieldValue| p.field.name == param.name)
            {
              if !field.value.is_array() {
                field.value = Value::Array(vec![field.value.clone()]);
              } else {
                field.value.as_array_mut().unwrap().push(value);
              }
            } else {
              query_params.push(param.clone().with_value(Value::Array(vec![value])));
            }
            continue;
          }

          let Ok(value) = param.ty.coerce_str(value) else {
            warn!("Failed to coerce query param {} to {}", name, param.ty);
            return Err(HttpError::InvalidParameter(param.name.clone()));

          };

          query_params.push(param.clone().with_value(value));
        }
      }
    }

    Ok(Some((path_params, query_params)))
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use pretty_assertions::assert_eq;

  use super::*;

  #[test_logger::test]
  fn test_parse() -> Result<()> {
    let route = Route::parse("/api/v1/users/{id:u32}/posts/{post_id:string}?filter:string[]&sort:string")?;
    assert_eq!(
      route.path_parts,
      vec![
        PathPart::Literal("api".to_owned()),
        PathPart::Literal("v1".to_owned()),
        PathPart::Literal("users".to_owned()),
        PathPart::Param(Field::new("id", Type::U32)),
        PathPart::Literal("posts".to_owned()),
        PathPart::Param(Field::new("post_id", Type::String)),
      ],
    );

    assert_eq!(
      route.query_params,
      vec![
        Field::new(
          "filter",
          Type::List {
            ty: Box::new(Type::String)
          }
        ),
        Field::new("sort", Type::String),
      ]
    );

    Ok(())
  }

  #[test_logger::test]
  fn regression_test_single_query() -> Result<()> {
    let route = Route::parse("/api?filter:string")?;
    assert_eq!(route.path_parts, vec![PathPart::Literal("api".to_owned()),],);

    assert_eq!(route.query_params, vec![Field::new("filter", Type::String),]);

    Ok(())
  }

  #[test_logger::test]
  fn test_no_query_string() -> Result<()> {
    let route = Route::parse("/api/v1/users/{id:u32}/posts/{post_id:string}")?;
    assert_eq!(
      route.path_parts,
      vec![
        PathPart::Literal("api".to_owned()),
        PathPart::Literal("v1".to_owned()),
        PathPart::Literal("users".to_owned()),
        PathPart::Param(Field::new("id", Type::U32)),
        PathPart::Literal("posts".to_owned()),
        PathPart::Param(Field::new("post_id", Type::String)),
      ],
    );

    assert_eq!(route.query_params, vec![]);

    Ok(())
  }

  #[test_logger::test]
  fn test_match_array() -> Result<()> {
    let route = Route::parse("/api/v1/users/{id:u32}/posts/{post_id:string}?filter:string[]&sort:string")?;

    assert_eq!(
      route.compare("/api/v1/users/123/posts/abc", Some("filter=foo&filter=bar&sort=asc"))?,
      Some((
        vec![
          Field::new("id", Type::U32).with_value(123),
          Field::new("post_id", Type::String).with_value("abc"),
        ],
        vec![
          Field::new(
            "filter",
            Type::List {
              ty: Box::new(Type::String)
            }
          )
          .with_value(vec!["foo".to_owned(), "bar".to_owned()]),
          Field::new("sort", Type::String).with_value("asc"),
        ],
      ))
    );

    Ok(())
  }

  #[test_logger::test]
  fn test_match() -> Result<()> {
    let route = Route::parse("/api/v1/users/{id:u32}/posts/{post_id:string}?filter:string&sort:string")?;

    assert_eq!(
      route.compare("/api/v1/users/123/posts/abc", Some("filter=foo&sort=asc"))?,
      Some((
        vec![
          Field::new("id", Type::U32).with_value(123),
          Field::new("post_id", Type::String).with_value("abc"),
        ],
        vec![
          Field::new("filter", Type::String).with_value("foo".to_owned()),
          Field::new("sort", Type::String).with_value("asc"),
        ],
      ))
    );

    Ok(())
  }
}
