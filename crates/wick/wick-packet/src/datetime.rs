use chrono::{DateTime as ChronoDateTime, TimeZone};

pub type DateTime = chrono::DateTime<chrono::Utc>;

/// Parse a date from a string in RFC3339 format, or in the format `%Y-%m-%d %H:%M:%S`.
pub fn parse_date(v: &str) -> Result<DateTime, crate::Error> {
  ChronoDateTime::parse_from_rfc3339(v)
    .map_err(|_| crate::Error::ParseDate(v.to_owned()))
    .map(|v| v.into())
}

/// Parse a UTC date from milliseconds since UNIX_EPOCH.
pub fn date_from_millis(millis: u64) -> Result<DateTime, crate::Error> {
  let dt = chrono::NaiveDateTime::from_timestamp_opt(millis as i64 / 1000, (millis % 1000 * 1_000_000) as u32)
    .ok_or(crate::Error::ParseDateMillis(millis))?;
  Ok(chrono::Utc.from_utc_datetime(&dt))
}

pub mod serde {

  use super::*;
  pub fn from_str_or_integer<'de, D>(deserializer: D) -> Result<DateTime, D::Error>
  where
    D: ::serde::Deserializer<'de>,
  {
    struct DateTimeVisitor;

    impl<'de> ::serde::de::Visitor<'de> for DateTimeVisitor {
      type Value = DateTime;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a connection target definition")
      }

      fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
      where
        E: ::serde::de::Error,
      {
        parse_date(s).map_err(|e| ::serde::de::Error::custom(e.to_string()))
      }

      fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
      where
        E: ::serde::de::Error,
      {
        date_from_millis(v).map_err(|e| ::serde::de::Error::custom(e.to_string()))
      }

      fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
      where
        E: ::serde::de::Error,
      {
        date_from_millis(v as _).map_err(|e| ::serde::de::Error::custom(e.to_string()))
      }
    }

    deserializer.deserialize_any(DateTimeVisitor)
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use chrono::Datelike;

  use super::*;
  use crate::Packet;

  #[test]
  fn test_datetime() -> Result<()> {
    let date_str = "2023-04-12T22:10:57+02:00";
    let date = parse_date(date_str)?;
    assert_eq!(date.year(), 2023);
    assert_eq!(date.month(), 4);

    Ok(())
  }

  #[test]
  fn test_datetime_packet() -> Result<()> {
    let date_str = "2023-04-12T22:10:57+02:00";
    let date = parse_date(date_str)?;

    let packet = Packet::encode(".", date_str);
    println!("date as_string packet: {:?}", packet);
    let actual: DateTime = packet.decode()?;
    assert_eq!(date, actual);

    let packet = Packet::encode(".", date);
    println!("date as_date packet: {:?}", packet);
    let actual: DateTime = packet.decode()?;
    assert_eq!(date, actual);

    Ok(())
  }

  #[test]
  fn test_from_int() -> Result<()> {
    #[derive(::serde::Deserialize, Debug, PartialEq)]
    struct Test {
      #[serde(deserialize_with = "serde::from_str_or_integer")]
      date: DateTime,
    }

    let expected = Test {
      date: parse_date("2021-04-12T22:10:57+02:00")?,
    };
    let json = serde_json::json!({"date": expected.date.timestamp_millis()});
    let actual: Test = serde_json::from_value(json)?;
    assert_eq!(expected, actual);

    let json = serde_json::json!({"date": "2021-04-12T22:10:57+02:00"});
    let actual: Test = serde_json::from_value(json)?;
    assert_eq!(expected, actual);

    Ok(())
  }
}
