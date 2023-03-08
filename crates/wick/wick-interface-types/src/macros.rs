#[macro_use]
macro_rules! kv_impl {
  ($t:ty) => {
    kv_impl!{$t, pub(self)}
  };
  ($t:ty, $v:vis) => {
    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Get the value for the requested field.
    $v fn get<K: AsRef<str>>(&self, field: K) -> Option<&$t> {
      self.0.get(field.as_ref())
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Get the value for the requested field.
    $v fn get_mut<K: AsRef<str>>(&mut self, field: K) -> Option<&mut $t> {
      self.0.get_mut(field.as_ref())
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Get the value for the requested field.
    $v fn contains_key<K: AsRef<str>>(&self, field: K) -> bool {
      self.0.contains_key(field.as_ref())
    }

    /// Insert a $t into the inner map.
    #[allow(unused, unreachable_pub)]
    $v fn insert<K: AsRef<str>>(&mut self, field: K, value: $t) {
      self.0.insert(field.as_ref().to_owned(), value);
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Remove a value from the inner Map.
    $v fn remove(&mut self, key:&str) -> Option<$t> {
      self.0.remove(key)
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Return a list of names in the inner HashMap.
    $v fn names(&self) -> Vec<String> {
      self.0.keys().cloned().collect()
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Return true if the inner HashMap is empty.
    $v fn is_empty(&self) -> bool {
      self.0.is_empty()
    }

    /// Return the inner HashMap.
    #[must_use]
    #[allow(unused, unreachable_pub)]
    $v fn into_inner(self) -> HashMap<String, $t> {
      self.0
    }

    /// Return a reference to the inner HashMap.
    #[must_use]
    #[allow(unused, unreachable_pub)]
    $v fn inner(&self) -> &HashMap<String, $t> {
      &self.0
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Returns the number of fields in the map.
    $v fn len(&self) -> usize {
      self.0.len()
    }
  };
}
