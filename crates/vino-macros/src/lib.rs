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

#[cfg(test)]
mod test {}
