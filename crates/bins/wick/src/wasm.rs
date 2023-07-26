use std::str::FromStr;

#[derive(Clone, Debug)]
struct StringPair(String, String);

impl FromStr for StringPair {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.split_once(':')
      .map(|(to, from)| StringPair(to.to_owned(), from.to_owned()))
      .ok_or_else(|| anyhow!("WASI directories need to be string pairs split by a colon, e.g. /to/dir:/from/dir"))
  }
}
