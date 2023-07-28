use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use flow_component::SharedComponent;
use tracing::Span;
use wick_config::config::UninitializedConfiguration;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_packet::RuntimeConfig;
use wick_test::{ComponentFactory, TestSuite};

#[test_logger::test(tokio::test)]
async fn baseline_component() -> Result<()> {
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let manifest = crate_dir.join("../../crates/integration/test-baseline-component/component.yaml");

  let fetch_options = wick_config::FetchOptions::default();

  let mut root_manifest = WickConfiguration::fetch(manifest.to_string_lossy(), fetch_options).await?;
  root_manifest.set_root_config(Some(RuntimeConfig::from(HashMap::from([(
    "default_err".into(),
    "err".into(),
  )]))));
  let root_manifest = root_manifest.finish()?.try_component_config()?;

  let mut suite = TestSuite::from_configuration(root_manifest.tests())?;
  let manifest = root_manifest.clone();

  let factory: ComponentFactory = Box::new(move |config| {
    let mut builder = UninitializedConfiguration::new(WickConfiguration::Component(manifest.clone()));

    let task = async move {
      builder.set_root_config(config);
      let manifest = builder
        .finish()
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?
        .try_component_config()
        .unwrap();
      let mut host = ComponentHostBuilder::default()
        .manifest(manifest)
        .span(Span::current())
        .build()
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?;
      host
        .start_runtime(None)
        .await
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?;
      let component: SharedComponent = Arc::new(wick_host::HostComponent::new(host));
      Ok(component)
    };
    Box::pin(task)
  });

  let runners = suite.run(factory, Default::default()).await?;

  for harness in runners {
    harness.print();
    let num_failed = harness.num_failed();
    if num_failed > 0 {
      return Err(anyhow::anyhow!("{} tests failed", num_failed));
    }
  }

  Ok(())
}
