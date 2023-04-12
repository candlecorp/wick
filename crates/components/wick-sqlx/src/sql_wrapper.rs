use sqlx::{Any, Database};
use wick_packet::TypeWrapper;

#[derive(Debug, Clone)]
pub(crate) struct SqlWrapper(pub(crate) TypeWrapper);

impl sqlx::Type<Any> for SqlWrapper {
  fn type_info() -> <Any as Database>::TypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("unknown").into()
  }
}
