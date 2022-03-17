use std::time::{Instant, SystemTime};

use serde::Serializer;

pub(crate) fn approx_instant<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_newtype_struct("SystemTime", &approx(instant))
}

pub(crate) fn approx_opt_instant<S>(instant: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  match instant {
    Some(i) => serializer.serialize_some(&approx(i)),
    None => serializer.serialize_none(),
  }
}

fn approx(instant: &Instant) -> SystemTime {
  let system_now = SystemTime::now();
  let instant_now = Instant::now();
  system_now - (instant_now - *instant)
}
