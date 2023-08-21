use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use serde_json::Value;
use sqlx::postgres::{PgRow, PgValueRef};
use sqlx::{Column, Decode, Postgres, Row, TypeInfo, ValueRef};

/// Can be used with serialize_with
pub(crate) fn serialize_valueref<S>(value: &PgValueRef, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  if value.is_null() {
    return s.serialize_none();
  }

  let value = value.clone();
  let info = value.type_info();
  let name = info.name();
  match name {
    "BOOL" => {
      let v: bool = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_bool(v)
    }
    "INT2" => {
      let v: i16 = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_i16(v)
    }
    "INT4" => {
      let v: i32 = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_i32(v)
    }
    "INT8" => {
      let v: i64 = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_i64(v)
    }
    "FLOAT4" => {
      let v: f32 = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_f32(v)
    }
    "FLOAT8" | "NUMERIC" => {
      let v: f64 = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_f64(v)
    }
    "CHAR" | "VARCHAR" | "TEXT" | "\"CHAR\"" => {
      let v: String = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_str(&v)
    }
    "BYTEA" => {
      let v: Vec<u8> = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_some(&v)
    }
    "JSON" | "JSONB" => {
      let v: Value = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_some(&v)
    }
    "TIMESTAMP" => {
      let v: wick_packet::DateTime = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_str(&v.to_rfc3339())
    }
    "TIMESTAMPTZ" => {
      let v: wick_packet::DateTime = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_str(&v.to_rfc3339())
    }
    "UUID" => {
      let v: String = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_str(&v)
    }
    _ => {
      let v: String = Decode::<Postgres>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_str(&v)
    }
  }
}

/// Can be used with serialize_with
pub(crate) fn serialize_row_as_vec<S>(x: &PgRow, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let cols = x.columns();
  let mut seq = s.serialize_seq(Some(cols.len()))?;
  for c in cols {
    let c: PgValueRef = x.try_get_raw(c.ordinal()).unwrap();
    let c = SerPgValueRef(c);
    seq.serialize_element(&c)?;
  }
  seq.end()
}

/// Can be used with serialize_with
pub(crate) fn serialize_row_as_map<S>(x: &PgRow, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let cols = x.columns();
  let mut map = s.serialize_map(Some(cols.len()))?;
  for col in cols {
    let c: PgValueRef = x.try_get_raw(col.ordinal()).unwrap();
    let c = SerPgValueRef(c);
    map.serialize_entry(col.name(), &c)?;
  }
  map.end()
}

#[derive(Serialize)]
pub(crate) struct SerVecPgRow(#[serde(serialize_with = "serialize_row_as_vec")] PgRow);

#[derive(Serialize)]
pub(crate) struct SerMapRow(#[serde(serialize_with = "serialize_row_as_map")] PgRow);

impl From<PgRow> for SerMapRow {
  fn from(row: PgRow) -> Self {
    SerMapRow(row)
  }
}

/// SerPgValueRef::from(pg_value_ref) will make your value serialize as its closest serde type.
#[derive(Serialize)]
pub(crate) struct SerPgValueRef<'r>(#[serde(serialize_with = "serialize_valueref")] PgValueRef<'r>);

impl From<PgRow> for SerVecPgRow {
  fn from(row: PgRow) -> Self {
    SerVecPgRow(row)
  }
}

#[cfg(test)]
mod integration_test {
  use anyhow::Result;
  use sqlx::{Connection, Executor, PgConnection};

  use super::*;

  fn read_row(row: &PgRow) -> Vec<Value> {
    let columns = row.columns();
    let mut result: Vec<Value> = Vec::with_capacity(columns.len());
    for c in columns {
      let value = row.try_get_raw(c.ordinal()).unwrap();
      let value = SerPgValueRef(value);
      let value = serde_json::to_value(&value).unwrap();
      result.push(value);
    }
    result
  }

  #[test_logger::test(tokio::test)]
  async fn it_works() -> Result<()> {
    let docker_host = std::env::var("TEST_HOST").unwrap();
    let pg_host = docker_host.split(':').next().unwrap();
    let password = std::env::var("TEST_PASSWORD").unwrap();
    let port = std::env::var("POSTGRES_PORT").unwrap();
    let conn_string = format!("postgres://postgres:{}@{}:{}/wick_test", password, pg_host, port);
    info!("connection string {}", conn_string);

    let mut conn = PgConnection::connect(&conn_string).await.unwrap();
    let row = conn.fetch_one("SELECT NOW()").await.unwrap();
    let row = read_row(&row);
    chrono::DateTime::parse_from_rfc3339(row[0].as_str().unwrap()).unwrap();

    let row = conn
      .fetch_one("select '00000000-0000-0000-0000-000000000000'::uuid")
      .await
      .unwrap();
    let row = read_row(&row);
    assert_eq!(row[0].as_str().unwrap(), "00000000-0000-0000-0000-000000000000");

    let row = conn.fetch_one("select 3.3").await.unwrap();
    let row = read_row(&row);
    assert_eq!(row[0].as_f64().unwrap(), 3.3);

    let row = conn.fetch_one("select 3.3::numeric(19,4)").await.unwrap();
    let row = read_row(&row);
    assert_eq!(row[0].as_f64().unwrap(), 3.3);

    let row = conn.fetch_one("select 'null'::jsonb").await.unwrap();
    let row = read_row(&row);
    assert_eq!(row[0], Value::Null);

    let row = conn.fetch_one("select 1 as foo, 'hello' as bar").await.unwrap();
    let row = SerMapRow::from(row);
    let row = serde_json::to_string(&row).unwrap();
    assert_eq!(row, r#"{"foo":1,"bar":"hello"}"#);

    let row = conn.fetch_one("select 1 as foo, 'hello' as bar").await.unwrap();
    let row = SerVecPgRow::from(row);
    let row = serde_json::to_string(&row).unwrap();
    assert_eq!(row, r#"[1,"hello"]"#);
    Ok(())
  }
}
