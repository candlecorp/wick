mod observer;
mod test_collection;

pub use observer::JsonWriter;
pub use test_collection::TestCollection;

#[macro_export]
macro_rules! interp {
  ($path:expr, $op:expr, $stream:expr) => {{
    use tokio_stream::StreamExt;
    use wasmflow_entity::Entity;
    use wasmflow_interpreter::{HandlerMap, Interpreter, InterpreterOptions, NamespaceHandler};
    use wasmflow_packet_stream::Invocation;
    const OPTIONS: Option<InterpreterOptions> = Some(InterpreterOptions {
      error_on_hung: true,
      // TODO: improve logic to ensure no remaining packets are sent after completion.
      // Turn this on to make tests fail in these cases.
      error_on_missing: false,
    });
    let def = wasmflow_manifest::WasmflowManifest::load_from_file($path)?;
    let network = from_def(&def)?;
    let collections = HandlerMap::new(vec![NamespaceHandler::new(
      "test",
      Box::new(test::TestCollection::new()),
    )]);
    let invocation = Invocation::new(Entity::test("test"), Entity::local($op), None);
    let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
    interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;
    let stream = interpreter.invoke(invocation, $stream).await?;
    let outputs: Vec<_> = stream.collect().await;
    println!("{:#?}", outputs);
    (interpreter, outputs)
  }};
}
