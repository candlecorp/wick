use chrono::DateTime as ChronoDateTime;

pub type DateTime = chrono::DateTime<chrono::Utc>;

/// Parse a date from a string in RFC3339 format, or in the format `%Y-%m-%d %H:%M:%S`.
pub fn parse_date(v: &str) -> Result<DateTime, crate::Error> {
  ChronoDateTime::parse_from_rfc3339(v)
    .map_err(|_| crate::Error::ParseDate(v.to_owned()))
    .map(|v| v.into())
}

/// Parse a UTC date from milliseconds since UNIX_EPOCH.
pub fn date_from_millis(millis: u64) -> Result<DateTime, crate::Error> {
  Ok(ChronoDateTime::from_utc(
    chrono::NaiveDateTime::from_timestamp_opt(millis as i64 / 1000, (millis % 1000 * 1_000_000) as u32)
      .ok_or(crate::Error::ParseDateMillis(millis))?,
    chrono::Utc,
  ))
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use chrono::Datelike;

  use super::*;

  #[test]
  fn test_datetime() -> Result<()> {
    let date_str = "2023-04-12T22:10:57+02:00";
    let date = parse_date(date_str)?;
    assert_eq!(date.year(), 2023);
    assert_eq!(date.month(), 4);

    Ok(())
  }
}
