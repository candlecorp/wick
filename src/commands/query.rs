use std::io::{self, BufRead};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::Args;
use markup_converter::{Format, Transcoder};
use serde_json::json;
use structured_output::StructuredOutput;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// Option to print raw output.
  #[clap(short = 'r', long = "raw", action)]
  raw_output: bool,

  /// The markup kind if there is no extension.
  #[clap(short = 't', long = "type", action)]
  kind: Option<MarkupKind>,

  /// Path to JSON, YAML, or TOML file.
  #[clap(short = 'f', long = "file", action)]
  path: Option<PathBuf>,

  /// The template.
  #[clap(action)]
  template: String,
}

#[derive(Debug, Clone)]
enum MarkupKind {
  Json,
  Toml,
  Yaml,
}

impl FromStr for MarkupKind {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "json" => Ok(MarkupKind::Json),
      "toml" => Ok(MarkupKind::Toml),
      "yaml" => Ok(MarkupKind::Yaml),
      _ => Err(anyhow!("Markup kind '{}' not supported", s)),
    }
  }
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let input = if let Some(path) = opts.path {
    match opts.kind {
      None => Transcoder::from_path(&path)?.to_json()?,
      Some(kind) => {
        let source = crate::io::read_to_string(&path).await?;
        match kind {
          MarkupKind::Json => Transcoder::new(Format::json(&source)?)?.to_json()?,
          MarkupKind::Toml => Transcoder::new(Format::toml(&source)?)?.to_json()?,
          MarkupKind::Yaml => Transcoder::new(Format::yaml(&source)?)?.to_json()?,
        }
      }
    }
  } else {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No file path passed, reading from <STDIN>. Use -f/--file to pass a file as an argument.");
    }
    let reader = io::BufReader::new(io::stdin());
    let lines = reader.lines();
    let markup = lines.collect::<Result<String, _>>()?;
    match opts.kind {
      Some(MarkupKind::Json) | None => Transcoder::new(Format::json(&markup)?)?.to_json()?,
      Some(MarkupKind::Toml) => Transcoder::new(Format::toml(&markup)?)?.to_json()?,
      Some(MarkupKind::Yaml) => Transcoder::new(Format::yaml(&markup)?)?.to_json()?,
    }
  };
  let _enter = span.enter();
  let trimmed = opts.template.trim();

  // If the passed template includes '{{' then we can assume it's a raw template.
  let template = if trimmed.contains("{{") {
    opts.template
  } else {
    // The trim_start_matches('.') removes a leading '.' if it exists.
    // This allows us to process a jq-style template as well as liquid-style templates.
    // Finally, wrap it in double braces to make it a liquid template.
    format!("{{{{ {} }}}}", trimmed.trim_start_matches('.'))
  };

  let template = liquid::ParserBuilder::with_stdlib().build()?.parse(&template)?;

  let globals = match input {
    serde_json::Value::Object(map) => liquid::model::to_object(&map)?,
    _ => {
      liquid::object!({
          "data": input
      })
    }
  };

  let result = template.render(&globals)?;
  let json = json!({"results":result});

  Ok(StructuredOutput::new(result, json))
}
