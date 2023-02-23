#[macro_export]
macro_rules! assert_equal {
    ($left:expr, $right:expr) => {{
        use $crate::k9::__macros__::colored::*;
        $crate::k9::assertions::initialize_colors();
        $crate::k9::config::set_panic(false);
        let args_str = format!(
            "{}, {}",
            stringify!($left).red(),
            stringify!($right).green(),
        );

        match  (&$left, &$right) {
            (left, right) => {
                let fail = *left != *right;
                let assertion = $crate::k9::make_assertion!(
                    "assert_equal",
                    args_str,
                    $crate::k9::assertions::equal::assert_equal(left, right, fail),
                    None,
                );
                if let Some(assertion) = assertion {
                  $crate::anyhow::bail!(assertion.failure_message);
                }
            }
        }
    }};
    ($left:expr, $right:expr, $($description:expr),*) => {{
        use $crate::k9::__macros__::colored::*;
        $crate::k9::assertions::initialize_colors();
        $crate::k9::config::set_panic(false);
        let description = format!($( $description ),*);
        let args_str = format!(
            "{}, {}, {}",
            stringify!($left).red(),
            stringify!($right).green(),
            stringify!($( $description ),* ).dimmed(),
        );
        match  (&$left, &$right) {
            (left, right) => {
                let fail = *left != *right;
                let assertion = $crate::k9::make_assertion!(
                    "assert_equal",
                    args_str,
                    $crate::k9::assertions::equal::assert_equal(left, right, fail),
                    Some(&description),
                );
                if let Some(assertion) = assertion {
                  $crate::anyhow::bail!(assertion.failure_message);
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! assert_true {
    ($left:expr) => {{
        use $crate::k9::__macros__::colored::*;
        $crate::k9::assertions::initialize_colors();
        $crate::k9::config::set_panic(false);
        let args_str = format!(
            "{}, {}",
            stringify!($left).red(),
            stringify!($right).green(),
        );

        match  (&$left) {
            (left) => {
                let fail = !*left;
                let assertion = $crate::k9::make_assertion!(
                    "assert_equal",
                    args_str,
                    $crate::k9::assertions::equal::assert_equal(left, true, fail),
                    None,
                );
                if let Some(assertion) = assertion {
                  $crate::anyhow::bail!(assertion.failure_message);
                }
            }
        }
    }};
    ($left:expr, $($description:expr),*) => {{
        use $crate::k9::__macros__::colored::*;
        $crate::k9::assertions::initialize_colors();
        $crate::k9::config::set_panic(false);
        let description = format!($( $description ),*);
        let args_str = format!(
            "{}, {}, {}",
            stringify!($left).red(),
            stringify!(true).green(),
            stringify!($( $description ),* ).dimmed(),
        );
        match  (&$left) {
            (left) => {
                let fail = !*left;
                let assertion = $crate::k9::make_assertion!(
                    "assert_equal",
                    args_str,
                    $crate::k9::assertions::equal::assert_equal(left, true, fail),
                    Some(&description),
                );
                if let Some(assertion) = assertion {
                  $crate::anyhow::bail!(assertion.failure_message);
                }
            }
        }
    }};
}
