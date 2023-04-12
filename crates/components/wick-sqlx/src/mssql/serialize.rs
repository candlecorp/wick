use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use sqlx::mssql::{MssqlRow, MssqlValueRef};
use sqlx::{Column, Decode, Mssql, Row, TypeInfo, ValueRef};

/// Can be used with serialize_with
pub(crate) fn serialize_mssql_value_ref<S>(value: &MssqlValueRef, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let value = value.clone();
  let info = value.type_info();
  let name = info.name();
  info!(name=%name,"type");
  match name {
    "BOOL" => {
      let v: bool = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_bool(v)
    }
    "TINYINT" => {
      let v: i8 = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_i8(v)
    }
    "SMALLINT" => {
      let v: i16 = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_i16(v)
    }
    "INT" => {
      let v: i32 = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_i32(v)
    }
    "BIGINT" => {
      let v: i64 = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_i64(v)
    }
    "FLOAT4" => {
      let v: f32 = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_f32(v)
    }
    "FLOAT8" | "NUMERIC" => {
      let v: f64 = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_f64(v)
    }
    "CHAR" | "BIGVARCHAR" | "VARCHAR" | "TEXT" | "\"CHAR\"" | "NVARCHAR" => {
      let v: String = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_str(&v)
    }
    // "BYTEA" => {
    //     let v: Vec<u8> = Decode::<Mssql>::decode(value).unwrap();
    //     s.serialize_some(&v)
    // }
    // "JSON" | "JSONB" => {
    //     let v: Value = Decode::<Mssql>::decode(value).unwrap();
    //     s.serialize_some(&v)
    // }
    // "TIMESTAMP" => {
    //     let v: sqlx::types::chrono::NaiveDateTime = Decode::<Mssql>::decode(value).unwrap();
    //     let v = v.format("%Y-%m-%dT%H:%M:%S.%f").to_string();
    //     s.serialize_str(&v)
    // }
    // "TIMESTAMPTZ" => {
    //     use sqlx::types::chrono;
    //     let v: chrono::DateTime::<chrono::Utc> = Decode::<Mssql>::decode(value).unwrap();
    //     s.serialize_str(&v.to_rfc3339())
    // }
    "UUID" => {
      let v: String = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_str(&v)
    }
    _ => {
      warn!(name=%name,"unknown type");
      let v: String = Decode::<Mssql>::decode(value).unwrap();
      s.serialize_str(&v)
    }
  }
}

/// Can be used with serialize_with
pub(crate) fn serialize_mssql_row_as_vec<S>(x: &MssqlRow, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let cols = x.columns();
  let mut seq = s.serialize_seq(Some(cols.len()))?;
  for c in cols {
    let c: MssqlValueRef = x.try_get_raw(c.ordinal()).unwrap();
    let c = SerMssqlValueRef(c);
    seq.serialize_element(&c)?;
  }
  seq.end()
}

/// Can be used with serialize_with
pub(crate) fn serialize_mssql_row_as_map<S>(x: &MssqlRow, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let cols = x.columns();
  let mut map = s.serialize_map(Some(cols.len()))?;
  for col in cols {
    let c: MssqlValueRef = x.try_get_raw(col.ordinal()).unwrap();
    let c = SerMssqlValueRef(c);
    map.serialize_entry(col.name(), &c)?;
  }
  map.end()
}

/// SerVecMssqlRow::from(pg_row) will make your row serialize as a vector.
#[derive(Serialize)]
pub(crate) struct SerVecMssqlRow(#[serde(serialize_with = "serialize_mssql_row_as_vec")] MssqlRow);

/// SerMapMssqlRow::from(pg_row) will make your row serialize as a map.
/// If you have multiple columns with the same name, the last one will win.
#[derive(Serialize)]
pub(crate) struct SerMapMssqlRow(#[serde(serialize_with = "serialize_mssql_row_as_map")] MssqlRow);

impl From<MssqlRow> for SerMapMssqlRow {
  fn from(row: MssqlRow) -> Self {
    SerMapMssqlRow(row)
  }
}

impl std::ops::Deref for SerMapMssqlRow {
  type Target = MssqlRow;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for SerMapMssqlRow {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<SerMapMssqlRow> for MssqlRow {
  fn from(row: SerMapMssqlRow) -> Self {
    row.0
  }
}

/// SerMssqlValueRef::from(pg_value_ref) will make your value serialize as its closest serde type.
#[derive(Serialize)]
pub(crate) struct SerMssqlValueRef<'r>(#[serde(serialize_with = "serialize_mssql_value_ref")] MssqlValueRef<'r>);

impl From<MssqlRow> for SerVecMssqlRow {
  fn from(row: MssqlRow) -> Self {
    SerVecMssqlRow(row)
  }
}

impl std::ops::Deref for SerVecMssqlRow {
  type Target = MssqlRow;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for SerVecMssqlRow {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<SerVecMssqlRow> for MssqlRow {
  fn from(row: SerVecMssqlRow) -> Self {
    row.0
  }
}

#[cfg(test)]
mod integration_test {
  use anyhow::Result;
  use pretty_assertions::assert_eq;
  use serde_json::Value;
  use sqlx::{Connection, Executor, MssqlConnection};

  use super::*;

  fn read_row(row: &MssqlRow) -> Vec<Value> {
    let columns = row.columns();
    let mut result: Vec<Value> = Vec::with_capacity(columns.len());
    for c in columns {
      let value = row.try_get_raw(c.ordinal()).unwrap();
      let value = SerMssqlValueRef(value);
      let value = serde_json::to_value(&value).unwrap();
      result.push(value);
    }
    result
  }

  async fn connect() -> MssqlConnection {
    let docker_host = std::env::var("DOCKER_HOST").unwrap();
    let db_host = docker_host.split(':').next().unwrap();
    let password = std::env::var("TEST_PASSWORD").unwrap();
    let port = std::env::var("MSSQL_PORT").unwrap();
    let conn_string = format!("mssql://SA:{}@{}:{}/wick_test", password, db_host, port);
    info!("connection string {}", conn_string);

    MssqlConnection::connect(&conn_string).await.unwrap()
  }

  #[test_logger::test(tokio::test)]
  #[ignore]
  async fn test_time() -> Result<()> {
    let mut conn = connect().await;
    let row = conn.fetch_one("SELECT CURRENT_TIMESTAMP;").await.unwrap();
    let row = read_row(&row);
    chrono::DateTime::parse_from_rfc3339(row[0].as_str().unwrap()).unwrap();
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  #[ignore]
  async fn test_guid() -> Result<()> {
    let mut conn = connect().await;
    let row = conn
      .fetch_one("select cast('00000000-0000-0000-0000-000000000000' as UNIQUEIDENTIFIER)")
      .await
      .unwrap();
    let row = read_row(&row);
    assert_eq!(row[0].as_str().unwrap(), "00000000-0000-0000-0000-000000000000");
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_int() -> Result<()> {
    let mut conn = connect().await;
    let row = conn.fetch_one("select cast(3 as integer);").await.unwrap();
    let row = read_row(&row);
    assert_eq!(row[0].as_i64().unwrap(), 3);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  #[ignore]
  async fn test_float() -> Result<()> {
    let mut conn = connect().await;

    let row = conn.fetch_one("select cast(3.3 as numeric(19,4));").await.unwrap();
    let row = read_row(&row);
    assert_eq!(row[0].as_f64().unwrap(), 3.3);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_map() -> Result<()> {
    // use sqlx::types::chrono;
    let mut conn = connect().await;

    let row = conn
      .fetch_one("select cast(1 as tinyint) as foo, cast('hello' as nvarchar(50)) as bar")
      .await
      .unwrap();
    let row = SerMapMssqlRow::from(row);
    let row = serde_json::to_string(&row).unwrap();
    assert_eq!(row, r#"{"foo":1,"bar":"hello"}"#);
    Ok(())

    // let row = conn.fetch_one("select cast(1 as foo, 'hello' as bar)").await.unwrap();
    // let row = SerVecMssqlRow::from(row);
    // let row = serde_json::to_string(&row).unwrap();
    // assert_eq!(row, r#"[1,"hello"]"#);
  }
}
