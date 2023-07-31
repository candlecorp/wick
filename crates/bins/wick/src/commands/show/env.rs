use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  _opts: Options,
  settings: wick_settings::Settings,
  _span: tracing::Span,
) -> Result<StructuredOutput> {
  let settings_text = serde_yaml::to_string(&settings)?;

  let xdg = wick_xdg::Settings::new();

  let env_text = serde_yaml::to_string(&xdg)?;

  let json = serde_json::json!({
    "settings": &settings,
    "env": &xdg,
  });

  let text = format!(
    r#"
# Settings
---
{settings_text}

# Environment
---
{env_text}
"#
  );

  let output = StructuredOutput::new(text, json);

  Ok(output)
}
