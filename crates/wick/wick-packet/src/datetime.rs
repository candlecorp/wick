pub use chrono::NaiveDateTime;

#[must_use]
/// Parse a date from a string in RFC3339 format, or in the format `%Y-%m-%d %H:%M:%S`.
pub fn parse_date(v: &str) -> NaiveDateTime {
  use chrono::DateTime;
  let v: DateTime<chrono::Utc> = DateTime::parse_from_rfc3339(v)
    .unwrap_or_else(|_| {
      let datetime = NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S %z").unwrap());
      DateTime::from_utc(
        datetime,
        chrono::TimeZone::offset_from_local_datetime(&chrono::Local, &datetime).unwrap(),
      )
    })
    .into();
  v.naive_utc()
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use chrono::Datelike;

  use super::*;

  #[test]
  fn test_datetime() -> Result<()> {
    let date = parse_date("2023-04-25 00:00:00");

    assert_eq!(date.year(), 2023);
    assert_eq!(date.month(), 4);
    let date = parse_date("2023-04-25 00:00:00 +02:00");

    assert_eq!(date.year(), 2023);
    assert_eq!(date.month(), 4);

    let date_str = "2023-04-12T22:10:57+02:00";
    let date = parse_date(date_str);
    assert_eq!(date.year(), 2023);
    assert_eq!(date.month(), 4);

    Ok(())
  }
}
