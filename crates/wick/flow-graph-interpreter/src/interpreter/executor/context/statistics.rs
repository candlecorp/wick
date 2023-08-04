use parking_lot::Mutex;
use performance_mark::Performance;
use uuid::Uuid;

#[derive(Debug)]
#[must_use]
pub(crate) struct ExecutionStatistics {
  #[allow(unused)]
  pub(crate) id: Uuid,
  pub(crate) performance: Mutex<Performance>,
}

impl ExecutionStatistics {
  pub(crate) fn new(uuid: Uuid) -> Self {
    Self {
      id: uuid,
      performance: Mutex::new(Default::default()),
    }
  }
  pub(crate) fn mark<T: AsRef<str>>(&self, label: T) {
    self.performance.lock().mark(label);
  }
  pub(crate) fn start<T: AsRef<str>>(&self, label: T) {
    self.performance.lock().start(label);
  }
  pub(crate) fn end(&mut self, label: &str) {
    let _ = self.performance.lock().end(label);
  }
  #[cfg(test)]
  pub(crate) fn print(&self) {
    let mut last = None;
    for event in self.performance.lock().events() {
      if last.is_none() {
        last = Some(event.instant());
      }
      let compare_to = last.unwrap();

      println!("{}: +{:?}", event.label(), event.instant().duration_since(compare_to));
      last = Some(event.instant());
    }
    for (name, period) in self.performance.lock().periods() {
      println!("{}: +{:?}", name, period.duration());
    }
  }
}
