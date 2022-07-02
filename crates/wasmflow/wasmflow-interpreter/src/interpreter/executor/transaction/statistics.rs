use parking_lot::Mutex;
use performance_mark::Performance;
use uuid::Uuid;

#[derive(Debug)]
#[must_use]
pub struct TransactionStatistics {
  pub id: Uuid,
  pub performance: Mutex<Performance>,
}

impl TransactionStatistics {
  pub fn new(uuid: Uuid) -> Self {
    Self {
      id: uuid,
      performance: Mutex::new(Default::default()),
    }
  }
  pub fn mark<T: AsRef<str>>(&self, label: T) {
    self.performance.lock().mark(label);
  }
  pub fn start<T: AsRef<str>>(&self, label: T) {
    self.performance.lock().start(label);
  }
  pub fn end(&mut self, label: &str) {
    let _ = self.performance.lock().end(label);
  }
  pub fn print(&self) {
    let mut last = None;
    for event in self.performance.lock().events() {
      if last == None {
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
