#[macro_export]
macro_rules! meh {
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
    #[allow(unreachable_code)]
    Ok::<_, crate::Error>($exp)
  };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! log_tap {
  ($expr:expr $(,)?) => {{
    let _e = $expr;
    log::trace!("{:?}", $expr);
    _e
  }};
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

#[cfg(test)]
mod test {}
