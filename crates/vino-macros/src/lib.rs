#[macro_export]
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
macro_rules! log_err {
  ($exp:expr) => {{
    log::error!("{}", $exp);
    Err($exp)
  }};
}

#[macro_export]
macro_rules! returns {
  ($type:ty) => {
    if (false) {
      return Err::<$type, crate::Error>("unused".into());
    }
  };
}

#[macro_export]
macro_rules! Ok {
  ($exp:expr) => {
    Ok::<_, crate::Error>($exp)
  };
}

#[allow(unused_macros)]
#[macro_export]
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
macro_rules! testlog {
    ($($arg:tt)+) => (
      if cfg!(test) {
        tracing::trace!($($arg)+)
      }
    )
}

pub use colored;

#[macro_export]
macro_rules! highlight {
    ($($arg:tt)+) => (
        use vino_macros::colored::Colorize;
        let indent = ">>>>>".to_owned().yellow().blink();
        let focus = ">>>>>>".to_owned().red().blink();
        let start = ">>>".to_owned().blue().blink();
        let end =   ">>>".to_owned().blue().dimmed();
        println!("{}\n{}\n{} {}\n{}\n{}", start,indent,focus,format!($($arg)+),indent,end);
    )
}

#[macro_export]
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
