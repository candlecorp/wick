use once_cell::sync::Lazy;

static CONFIG: Lazy<Config> = Lazy::new(|| {
  let db = wick_component::packet::CollectionLink::new("wick", "db");
  Config { db }
});

pub(crate) struct Config {
  pub(crate) db: wick_component::packet::CollectionLink,
}

pub(crate) fn get_config() -> &'static Config {
  unsafe { &CONFIG }
}
