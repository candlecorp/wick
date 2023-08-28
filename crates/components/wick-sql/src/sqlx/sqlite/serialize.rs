use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use sqlx::sqlite::{SqliteRow, SqliteValueRef};
use sqlx::{Column, Decode, Row, Sqlite, TypeInfo, Value, ValueRef};

/// Can be used with serialize_with
pub(crate) fn serialize_value_ref<S>(value: &SqliteValueRef, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  if value.is_null() {
    return s.serialize_none();
  }

  let info = value.type_info();
  let value = value.to_owned();
  let value = value.as_ref();

  let name = info.name();

  match name {
    "NULL" => s.serialize_none(),

    "TEXT" => {
      let v: String = Decode::<Sqlite>::decode(value).unwrap();
      s.serialize_str(&v)
    }
    "REAL" => {
      let v: f64 = Decode::<Sqlite>::decode(value).unwrap();
      s.serialize_f64(v)
    }
    "BLOB" => {
      let v: Vec<u8> = Decode::<Sqlite>::decode(value).unwrap();
      s.serialize_bytes(&v)
    }
    "INTEGER" => {
      let v: i64 = Decode::<Sqlite>::decode(value).unwrap();
      s.serialize_i64(v)
    }
    "NUMERIC" => {
      let v: f64 = Decode::<Sqlite>::decode(value).unwrap();
      s.serialize_f64(v)
    }
    "BOOLEAN" => {
      let v: bool = Decode::<Sqlite>::decode(value).unwrap();
      s.serialize_bool(v)
    }
    "DATE" => {
      let v: wick_packet::DateTime = Decode::<Sqlite>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_str(&v.to_rfc3339())
    }
    "TIME" => {
      unimplemented!("TIME not supported");
    }
    "DATETIME" => {
      let v: wick_packet::DateTime = Decode::<Sqlite>::decode(value).map_err(serde::ser::Error::custom)?;
      s.serialize_str(&v.to_rfc3339())
    }
    _ => {
      warn!(name=%name,"unknown type");
      let v: String = Decode::<Sqlite>::decode(value).unwrap();
      s.serialize_str(&v)
    }
  }
}

/// Can be used with serialize_with
pub(crate) fn serialize_row_as_vec<S>(x: &SqliteRow, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let cols = x.columns();
  let mut seq = s.serialize_seq(Some(cols.len()))?;
  for c in cols {
    let c: SqliteValueRef = x.try_get_raw(c.ordinal()).unwrap();
    let c = SerValueRef(c);
    seq.serialize_element(&c)?;
  }
  seq.end()
}

/// Can be used with serialize_with
pub(crate) fn serialize_row_as_map<S>(x: &SqliteRow, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let cols = x.columns();
  let mut map = s.serialize_map(Some(cols.len()))?;
  for col in cols {
    let c = x.try_get_raw(col.ordinal()).unwrap();
    let c = SerValueRef(c);
    map.serialize_entry(col.name(), &c)?;
  }
  map.end()
}

#[derive(Serialize)]
pub(crate) struct SerVecRow(#[serde(serialize_with = "serialize_row_as_vec")] SqliteRow);

#[derive(Serialize)]
pub(crate) struct SerMapRow(#[serde(serialize_with = "serialize_row_as_map")] SqliteRow);

impl From<SqliteRow> for SerMapRow {
  fn from(row: SqliteRow) -> Self {
    SerMapRow(row)
  }
}

#[derive(Serialize)]
pub(crate) struct SerValueRef<'r>(#[serde(serialize_with = "serialize_value_ref")] SqliteValueRef<'r>);

impl From<SqliteRow> for SerVecRow {
  fn from(row: SqliteRow) -> Self {
    SerVecRow(row)
  }
}

#[cfg(test)]
mod integration_test {
  use anyhow::Result;
  use pretty_assertions::assert_eq;
  use serde_json::Value;
  use sqlx::{Connection, Executor, SqliteConnection};

  use super::*;

  fn read_row(row: &SqliteRow) -> Vec<Value> {
    let columns = row.columns();
    let mut result: Vec<Value> = Vec::with_capacity(columns.len());
    for c in columns {
      let value = row.try_get_raw(c.ordinal()).unwrap();
      let value = SerValueRef(value);
      let value = serde_json::to_value(&value).unwrap();
      result.push(value);
    }
    result
  }

  async fn connect() -> SqliteConnection {
    let db = std::env::var("SQLITE_DB").unwrap();
    let conn_string = format!("file://{}", db);

    SqliteConnection::connect(&conn_string).await.unwrap()
  }

  #[test_logger::test(tokio::test)]
  async fn test_int() -> Result<()> {
    let mut conn = connect().await;
    let row = conn.fetch_one("select cast(3 as integer);").await?;
    let row = read_row(&row);
    assert_eq!(row[0].as_i64().unwrap(), 3);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_map() -> Result<()> {
    // use sqlx::types::chrono;
    let mut conn = connect().await;

    let row = conn
      .fetch_one("select cast(1 as tinyint) as foo, cast('hello' as nvarchar(50)) as bar")
      .await?;
    let row = SerMapRow::from(row);
    let row = serde_json::to_string(&row).unwrap();
    assert_eq!(row, r#"{"foo":1,"bar":"hello"}"#);
    Ok(())
  }
}
