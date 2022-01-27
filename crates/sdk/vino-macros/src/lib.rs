#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/71604398?s=200&v=4")]
#![doc = include_str!("../README.md")]

// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::if_then_some_else_none,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow()]

pub use tracing;

#[macro_export]
/// Test a condition and if it is false, return the supplied error
macro_rules! ensure {
    ($cond:expr $(,)?) => {
        $crate::ensure!(
            $cond,
            $crate::private::concat!("Condition failed: `", $crate::private::stringify!($cond), "`"),
        )
    };
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::Error::Other($msg.to_string()));
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return Err($err);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err(format!($fmt, $($arg)*));
        }
    };
}

#[macro_export]
/// Turns an expression into an error while logging it.
macro_rules! log_err {
  ($exp:expr) => {{
    tracing::error!("{}", $exp);
    Err($exp)
  }};
}

#[allow(unused_macros)]
#[macro_export]
/// Wrap an expression that prints debug output to the terminal while returning the original expression. Useful for logging without disturbing the code's structure.
///
/// ```
/// # use vino_macros::*;
/// # fn main() {
///   let vec = vec![1,2,3,4,5];
///   let doubled: Vec<_> = vec.iter().map(|i| log_tap!(i * 2)).collect();
/// # }
/// ```
macro_rules! log_tap {
  ($expr:expr $(,)?) => {{
    let _e = $expr;
    let indent = "]]]]";
    println!("{}\n{} {}\n{}", indent, indent, format!("{:?}", $expr), indent);

    _e
  }};
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[macro_export]
/// Aggressively prints to the terminal. Useful for rapid debugging in a sea of
/// terminal output.
///
/// ## Example
///
/// ```
/// # use vino_macros::*;
/// # fn main() {
///
/// let data = vec![1,2,3];
/// highlight!("{:?}", data);
/// # }
/// ```
macro_rules! highlight {
    ($($arg:tt)+) => (
      {
        let indent = ">>>>>";
        let focus = ">>>>>>";
        let start = ">>>";
        let end =   ">>>";
        println!("{}\n{}\n{} {}\n{}\n{}", start,indent,focus,format!($($arg)+),indent,end);
      }
    )
}

#[macro_export]
/// Returns an unwrapped Option if Some() otherwise returns the passed expression
///
/// ```
/// # use vino_macros::*;
/// # fn main() {
///
/// fn gen_msg(num: Option<i32>) -> String {
///   let num = some_or_bail!(num, "No number passed".to_owned());
///   format!("Num was {}", num)
/// }
///
/// let msg = gen_msg(Some(22));
/// println!("{}", msg);
/// # assert_eq!(msg, "Num was 22");
/// let msg = gen_msg(None);
/// println!("{}", msg);
/// # assert_eq!(msg, "No number passed");
/// # }
/// ```
macro_rules! some_or_bail {
  ($opt:expr, $ret:expr $(,)?) => {{
    match $opt {
      Some(stuff) => stuff,
      None => {
        return $ret;
      }
    }
  }};
}

#[macro_export]
/// Returns an unwrapped Option if Some() otherwise continues a loop.
///
/// ```
/// # use vino_macros::*;
/// # fn main() {
///
/// for i in vec![Some(1), None, Some(2)] {

///   println!("Starting loop");
///   let num = some_or_continue!(i);
///   println!("Got {}", num);
/// }
/// # }
/// ```
macro_rules! some_or_continue {
  ($opt:expr $(,)?) => {{
    match $opt {
      Some(stuff) => stuff,
      None => {
        continue;
      }
    }
  }};
}

#[macro_export]
/// Returns an unwrapped Ok if Ok() otherwise continues a loop.
///
/// ```
/// # use vino_macros::*;
/// # fn main() {
///
/// for i in vec![Ok(1), Err("Oh no"), Ok(2)] {

///   println!("Starting loop");
///   let num = ok_or_continue!(i);
///   println!("Got {}", num);
/// }
/// # }
/// ```
macro_rules! ok_or_continue {
  ($opt:expr $(,)?) => {{
    match $opt {
      Ok(stuff) => stuff,
      Err(e) => {
        tracing::debug!("Unexpected but recoverable error: {}", e.to_string());
        continue;
      }
    }
  }};
}

#[macro_export]
/// Returns an unwrapped Result if Ok() otherwise returns the passed expression
///
/// ```
/// # use vino_macros::*;
/// # fn main() {
/// fn generates_err() -> Result<i32, String>{ Err("Got an error".to_owned())}
/// fn generates_ok() -> Result<i32, String>{ Ok(42) }
///
/// fn do_work() -> i32 {
///   let num_work = ok_or_bail!(generates_err(), 0);
///   println!("Doing {} units of work...", num_work);
///   num_work
/// }
///
/// let work_done = do_work();
/// println!("Did {} units of work", work_done);
/// # assert_eq!(work_done, 0);
/// # }
/// ```
macro_rules! ok_or_bail {
  ($result:expr, $ret:expr $(,)?) => {{
    match $result {
      Ok(stuff) => stuff,
      Err(e) => {
        tracing::debug!("Unexpected but recoverable error: {}", e.to_string());
        return $ret;
      }
    }
  }};
}

lazy_static::lazy_static!(
  #[doc(hidden)]
  pub static ref START_TIMES: Arc<Mutex<HashMap<String, Instant>>> = {
    Arc::new(Mutex::new(HashMap::new()))
  };
);

#[macro_export]
#[doc(hidden)]
macro_rules! mark {
  () => {{
    let _ = $crate::START_TIMES.lock().and_then(|mut h| {
      h.insert($crate::function_path!(), std::time::Instant::now());
      let msg = format!("BENCH::mark:{}:{}", $crate::function_path!(), line!());
      println!("{}", msg);
      Ok(())
    });
  }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! elapsed {
  () => {{
    let _ = $crate::START_TIMES.lock().and_then(|h| {
      let time = h.get(&$crate::function_path!());
      let elapsed = time
        .map(|t| t.elapsed().as_micros().to_string())
        .unwrap_or("no start time marked...".to_owned());
      println!("BENCH::{}:{}: +{}μs", $crate::function_path!(), line!(), elapsed);
      Ok(())
    });
  }};
}
#[macro_export]
#[doc(hidden)]
macro_rules! function_path {
  () => {{
    fn f() {}
    fn type_name_of<T>(_: T) -> &'static str {
      std::any::type_name::<T>()
    }
    let name = type_name_of(f);
    name[..name.len() - 16].to_owned()
  }};
}
