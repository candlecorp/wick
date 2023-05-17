#[allow(clippy::expect_used)]
pub(super) fn setup(style: human_panic::PanicStyle) {
  match style {
    human_panic::PanicStyle::Debug => {}
    human_panic::PanicStyle::Human => {
      let meta = human_panic::Metadata {
        name: crate::BIN_NAME.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "wick authors".into(),
        homepage: "https://github.com/candlecorp/wick".into(),
      };

      std::panic::set_hook(Box::new(move |info: &std::panic::PanicInfo| {
        let file_path = human_panic::handle_dump(&meta, info);
        human_panic::print_msg(file_path, &meta).expect("human-panic: printing error message to console failed");
      }));
    }
  }
}
