# performance-mark

This library is for marking periods of time and events in those periods.

## Usage

```rust
use performance_mark::Performance;
use std::thread::sleep;
use std::time::Duration;

fn main() {
  let wait = Duration::from_millis(100);

  let mut perf = Performance::new();

  perf.mark("start");

  sleep(wait);

  perf.start("middle");
  sleep(wait);
  perf.end("middle");

  sleep(wait);

  perf.mark("end");

  println!("{:?}", perf.events());

  assert_eq!(perf.events().len(), 2);
  assert!(perf.events()[0] < perf.events()[1]);
  assert_eq!(perf.periods().len(), 1);
  assert!(perf.periods().get("middle").unwrap().duration() >= wait);
}
```
