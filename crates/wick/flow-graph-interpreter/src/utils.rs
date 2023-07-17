use wick_packet::Entity;

pub(crate) fn path_to_entity(path: &str) -> Entity {
  path
    .split_once("::")
    .map_or_else(|| Entity::local(path), |(path, op)| Entity::operation(path, op))
}

pub(crate) struct Bucket<T> {
  inner: std::sync::Arc<parking_lot::Mutex<Option<T>>>,
}

impl<T> Bucket<T> {
  pub(crate) fn new(inner: T) -> Self {
    Self {
      inner: std::sync::Arc::new(parking_lot::Mutex::new(Some(inner))),
    }
  }

  pub(crate) fn take(&self) -> Option<T> {
    self.inner.lock().take()
  }

  pub(crate) fn replace(&self, inner: T) -> Option<T> {
    self.inner.lock().replace(inner)
  }
}
