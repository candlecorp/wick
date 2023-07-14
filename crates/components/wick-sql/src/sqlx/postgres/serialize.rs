use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use serde_json::Value;
use sqlx::postgres::{PgRow, PgValueRef};
use sqlx::{Column, Decode, Postgres, Row, TypeInfo, ValueRef};

/// Can be used with serialize_with
pub(crate) fn serialize_pgvalueref<S>(value: &PgValueRef, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
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
        // PgType::Name => "NAME",
        // PgType::Oid => "OID",
        // PgType::JsonArray => "JSON[]",
        // PgType::Point => "POINT",
        // PgType::Lseg => "LSEG",
        // PgType::Path => "PATH",
        // PgType::Box => "BOX",
        // PgType::Polygon => "POLYGON",
        // PgType::Line => "LINE",
        // PgType::LineArray => "LINE[]",
        // PgType::Cidr => "CIDR",
        // PgType::CidrArray => "CIDR[]",
        // PgType::Unknown => "UNKNOWN",
        // PgType::Circle => "CIRCLE",
        // PgType::CircleArray => "CIRCLE[]",
        // PgType::Macaddr8 => "MACADDR8",
        // PgType::Macaddr8Array => "MACADDR8[]",
        // PgType::Macaddr => "MACADDR",
        // PgType::Inet => "INET",
        // PgType::BoolArray => "BOOL[]",
        // PgType::ByteaArray => "BYTEA[]",
        // PgType::CharArray => "\"CHAR\"[]",
        // PgType::NameArray => "NAME[]",
        // PgType::Int2Array => "INT2[]",
        // PgType::Int4Array => "INT4[]",
        // PgType::TextArray => "TEXT[]",
        // PgType::BpcharArray => "CHAR[]",
        // PgType::VarcharArray => "VARCHAR[]",
        // PgType::Int8Array => "INT8[]",
        // PgType::PointArray => "POINT[]",
        // PgType::LsegArray => "LSEG[]",
        // PgType::PathArray => "PATH[]",
        // PgType::BoxArray => "BOX[]",
        // PgType::Float4Array => "FLOAT4[]",
        // PgType::Float8Array => "FLOAT8[]",
        // PgType::PolygonArray => "POLYGON[]",
        // PgType::OidArray => "OID[]",
        // PgType::MacaddrArray => "MACADDR[]",
        // PgType::InetArray => "INET[]",
        // PgType::Date => "DATE",
        // PgType::Time => "TIME",
        // PgType::Timestamp => "TIMESTAMP",
        // PgType::TimestampArray => "TIMESTAMP[]",
        // PgType::DateArray => "DATE[]",
        // PgType::TimeArray => "TIME[]",
        // PgType::Timestamptz => "TIMESTAMPTZ",
        // PgType::TimestamptzArray => "TIMESTAMPTZ[]",
        // PgType::Interval => "INTERVAL",
        // PgType::IntervalArray => "INTERVAL[]",
        // PgType::NumericArray => "NUMERIC[]",
        // PgType::Timetz => "TIMETZ",
        // PgType::TimetzArray => "TIMETZ[]",
        // PgType::Bit => "BIT",
        // PgType::BitArray => "BIT[]",
        // PgType::Varbit => "VARBIT",
        // PgType::VarbitArray => "VARBIT[]",
        // PgType::Numeric => "NUMERIC",
        // PgType::Record => "RECORD",
        // PgType::RecordArray => "RECORD[]",
        // PgType::UuidArray => "UUID[]",
        // PgType::JsonbArray => "JSONB[]",
        // PgType::Int4Range => "INT4RANGE",
        // PgType::Int4RangeArray => "INT4RANGE[]",
        // PgType::NumRange => "NUMRANGE",
        // PgType::NumRangeArray => "NUMRANGE[]",
        // PgType::TsRange => "TSRANGE",
        // PgType::TsRangeArray => "TSRANGE[]",
        // PgType::TstzRange => "TSTZRANGE",
        // PgType::TstzRangeArray => "TSTZRANGE[]",
        // PgType::DateRange => "DATERANGE",
        // PgType::DateRangeArray => "DATERANGE[]",
        // PgType::Int8Range => "INT8RANGE",
        // PgType::Int8RangeArray => "INT8RANGE[]",
        // PgType::Jsonpath => "JSONPATH",
        // PgType::JsonpathArray => "JSONPATH[]",
        // PgType::Money => "MONEY",
        // PgType::MoneyArray => "MONEY[]",
        // PgType::Void => "VOID",
    }
}

/// Can be used with serialize_with
pub(crate) fn serialize_pgrow_as_vec<S>(x: &PgRow, s: S) -> Result<S::Ok, S::Error>
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
pub(crate) fn serialize_pgrow_as_map<S>(x: &PgRow, s: S) -> Result<S::Ok, S::Error>
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

/// SerVecPgRow::from(pg_row) will make your row serialize as a vector.
#[derive(Serialize)]
pub(crate) struct SerVecPgRow(#[serde(serialize_with = "serialize_pgrow_as_vec")] PgRow);

/// SerMapPgRow::from(pg_row) will make your row serialize as a map.
/// If you have multiple columns with the same name, the last one will win.
#[derive(Serialize)]
pub(crate) struct SerMapPgRow(#[serde(serialize_with = "serialize_pgrow_as_map")] PgRow);

impl From<PgRow> for SerMapPgRow {
  fn from(row: PgRow) -> Self {
    SerMapPgRow(row)
  }
}

impl std::ops::Deref for SerMapPgRow {
  type Target = PgRow;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for SerMapPgRow {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<SerMapPgRow> for PgRow {
  fn from(row: SerMapPgRow) -> Self {
    row.0
  }
}

/// SerPgValueRef::from(pg_value_ref) will make your value serialize as its closest serde type.
#[derive(Serialize)]
pub(crate) struct SerPgValueRef<'r>(#[serde(serialize_with = "serialize_pgvalueref")] PgValueRef<'r>);

impl From<PgRow> for SerVecPgRow {
  fn from(row: PgRow) -> Self {
    SerVecPgRow(row)
  }
}

impl std::ops::Deref for SerVecPgRow {
  type Target = PgRow;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for SerVecPgRow {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<SerVecPgRow> for PgRow {
  fn from(row: SerVecPgRow) -> Self {
    row.0
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
    let row = SerMapPgRow::from(row);
    let row = serde_json::to_string(&row).unwrap();
    assert_eq!(row, r#"{"foo":1,"bar":"hello"}"#);

    let row = conn.fetch_one("select 1 as foo, 'hello' as bar").await.unwrap();
    let row = SerVecPgRow::from(row);
    let row = serde_json::to_string(&row).unwrap();
    assert_eq!(row, r#"[1,"hello"]"#);
    Ok(())
  }
}
