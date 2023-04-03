use serde_json::Value;
use tokio_postgres::types::{accepts, FromSql, Type};

#[derive(Debug, Clone)]
struct SqlValue(Value);

impl<'a> FromSql<'a> for SqlValue {
  accepts! {VARCHAR, NUMERIC}

  fn from_sql(ty: &Type, mut raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
    let a = Self(FromSql::from_sql(ty, raw)?);
    println!("FROM_SQL: {:?}", a);
    Ok(a)
    // let value = match *ty {
    //   Type::VARCHAR => {
    //     let s = std::str::from_utf8(raw)?;
    //     Value::String(s.to_owned())
    //   }
    //   Type::NUMERIC => {
    //     let v = raw.read_i64::<BigEndian>()?;

    //     Value::Number(Number::from(v))
    //   }
    //   _ => todo!(),
    // };
    // println!("hey: {:?}", value);
    // Ok(SqlValue(value))
  }
}
