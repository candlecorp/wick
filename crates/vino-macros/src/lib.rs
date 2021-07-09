//! Macros used by the Vino project

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
    // clippy::too_many_lines,
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
    // dead_code,
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
    path_statements ,
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
    // unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    // missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow()]

#[macro_export]
/// TODO need to get rid of this
macro_rules! ok_or_log {
  ($expr:expr $(,)?) => {{
    match $expr {
      Ok(_) => {}
      Err(e) => {
        log::error!("Unexpected error: {}", e);
      }
    }
  }};
}

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
/// Wraps an error with a log statement
macro_rules! log_err {
  ($exp:expr) => {{
    log::error!("{}", $exp);
    Err($exp)
  }};
}

#[macro_export]
/// Turn an Ok(expression) into an Ok::<_, crate::Error>(expression).
/// Useful for quickly dismissing warnings that Rust can not infer a block's Error type
macro_rules! Ok {
  ($exp:expr) => {
    Ok::<_, crate::Error>($exp)
  };
}

#[allow(unused_macros)]
#[macro_export]
/// Wrap an expression that prints colorized debug output to the terminal
macro_rules! log_tap {
  ($expr:expr $(,)?) => {{
    let _e = $expr;
    use vino_macros::colored::Colorize;
    let indent = "]]]]".to_owned().blue().blink();
    let focus = "]]]]".to_owned().blue().blink();
    println!(
      "{}\n{} {}\n{}",
      indent,
      focus,
      format!("{:?}", $expr),
      indent
    );

    _e
  }};
}

#[macro_export]
/// Trace logging that only occurs during tests
macro_rules! testlog {
    ($($arg:tt)+) => (
      if cfg!(test) {
        tracing::trace!($($arg)+)
      }
    )
}

pub use colored;

#[macro_export]
/// Prints aggressively colorized output to the terminal. Useful for rapid debugging in a sea of
/// console output.
macro_rules! highlight {
    ($($arg:tt)+) => (
      {
        use vino_macros::colored::Colorize;
        let indent = ">>>>>".to_owned().yellow().blink();
        let focus = ">>>>>>".to_owned().red().blink();
        let start = ">>>".to_owned().blue().blink();
        let end =   ">>>".to_owned().blue().dimmed();
        println!("{}\n{}\n{} {}\n{}\n{}", start,indent,focus,format!($($arg)+),indent,end);
      }
    )
}

#[macro_export]
macro_rules! some_or_return {
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
macro_rules! ok_or_return {
  ($result:expr, $ret:expr $(,)?) => {{
    highlight!("result: {:?}", $result);
    highlight!("{:?}", $ret);
    match $result {
      Ok(stuff) => stuff,
      Err(_) => {
        return $ret;
      }
    }
  }};
}

#[macro_export]
/// TODO need to get rid of this
macro_rules! bail {
  ($expr:expr $(,)?) => {{
    match $expr {
      Ok(_) => {}
      Err(e) => {
        return e;
      }
    }
  }};
}
