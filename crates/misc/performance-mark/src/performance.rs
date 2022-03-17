use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::error::Error;

#[derive(Debug, Default)]
#[cfg_attr(feature = "derive_serde", derive(serde::Serialize))]
#[must_use]
pub struct Performance {
  pub events: Vec<PerformanceMark>,
  pub periods: HashMap<String, PerformancePeriod>,
}

impl Performance {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn mark<T: AsRef<str>>(&mut self, label: T) {
    self.events.push(PerformanceMark::new(label.as_ref().to_owned()));
  }

  pub fn start<T: AsRef<str>>(&mut self, label: T) {
    self.periods.insert(label.as_ref().to_owned(), PerformancePeriod::new());
  }

  pub fn end(&mut self, label: &str) -> Result<(), Error> {
    let period = self.periods.get_mut(label).ok_or(Error::EndBeforeStart)?;
    period.end();
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive_serde", derive(serde::Serialize))]
#[must_use]
pub struct PerformanceMark {
  pub label: String,
  #[cfg_attr(feature = "derive_serde", serde(serialize_with = "crate::serde::approx_instant"))]
  pub instant: Instant,
}

impl PerformanceMark {
  pub fn new(label: String) -> Self {
    Self {
      label,
      instant: Instant::now(),
    }
  }
}

impl PartialOrd for PerformanceMark {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.instant.partial_cmp(&other.instant)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "derive_serde", derive(serde::Serialize))]
#[must_use]
pub struct PerformancePeriod {
  #[cfg_attr(feature = "derive_serde", serde(serialize_with = "crate::serde::approx_instant"))]
  pub start: Instant,
  #[cfg_attr(feature = "derive_serde", serde(serialize_with = "crate::serde::approx_opt_instant"))]
  pub end: Option<Instant>,
  pub duration: Duration,
}

impl Default for PerformancePeriod {
  fn default() -> Self {
    Self::new()
  }
}

impl PerformancePeriod {
  pub fn new() -> Self {
    Self {
      start: Instant::now(),
      end: None,
      duration: Duration::new(0, 0),
    }
  }

  pub fn end(&mut self) {
    let now = Instant::now();
    self.end = Some(now);
    self.duration = now - self.start;
  }

  #[must_use]
  pub fn duration(&self) -> Duration {
    self.end.unwrap_or_else(Instant::now) - self.start
  }
}

impl PartialOrd for PerformancePeriod {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.duration().partial_cmp(&other.duration())
  }
}

#[cfg(test)]
mod tests {
  use std::thread::sleep;

  use anyhow::Result;

  use super::*;

  fn is_sync_send<T>()
  where
    T: Send + Sync,
  {
  }

  #[test]
  fn test_sync_send() {
    is_sync_send::<Performance>();
  }

  #[test]
  fn test() -> Result<()> {
    let wait = Duration::from_millis(100);
    let mut perf = Performance::new();
    perf.mark("start");
    sleep(wait);
    perf.start("middle");
    sleep(wait);
    perf.end("middle")?;
    sleep(wait);
    perf.mark("end");

    println!("{:?}", perf.events);
    assert_eq!(perf.events.len(), 2);
    assert!(perf.events[0] < perf.events[1]);
    assert_eq!(perf.periods.len(), 1);
    assert!(perf.periods.get("middle").unwrap().duration() >= wait);

    Ok(())
  }

  #[test]
  #[cfg(feature = "derive_serde")]
  fn test_serde() -> Result<()> {
    let mut perf = Performance::new();
    perf.mark("start");
    perf.start("middle");
    perf.end("middle")?;
    perf.mark("end");
    let json = serde_json::to_string_pretty(&perf)?;
    println!("{}", json);
    Ok(())
  }
}
