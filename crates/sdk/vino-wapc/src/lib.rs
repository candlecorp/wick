#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}

#[cfg(feature = "guest")]
pub mod exports;

pub mod signals;

pub use signals::*;
