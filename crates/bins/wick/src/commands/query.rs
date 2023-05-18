use std::io::{self, BufRead};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::Args;
use jaq_core::{parse, Ctx, Definitions, RcIter, Val};
use markup_converter::{Format, Transcoder};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct QueryCommand {
  /// Option to print raw output.
  #[clap(short = 'r', long = "raw", action)]
  raw_output: bool,

  /// Option to print raw output.
  #[clap(short = 't', long = "type", action)]
  kind: Option<MarkupKind>,

  /// Path to JSON, YAML, or TOML file.
  #[clap(short = 'f', long = "file", action)]
  path: Option<PathBuf>,

  /// The query.
  #[clap(action)]
  query: String,
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

pub(crate) async fn handle(opts: QueryCommand, _settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
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

  let filter = &opts.query;

  // parse the filter in the context of the given definitions
  let mut errs = Vec::new();
  let (filters, errors) = parse::parse(filter, parse::main());

  if !errors.is_empty() {
    for error in errors {
      error!("Error parsing query: {}", error);
    }
    return Err(anyhow!("Errors parsing queries"));
  }

  if let Some(filters) = filters {
    // start out only from core filters,
    // which do not include filters in the standard library
    // such as `map`, `select` etc.
    let defs = Definitions::core();

    let filters = defs.finish(filters, Vec::new(), &mut errs);
    let inputs = RcIter::new(core::iter::empty());

    // iterator over the output values
    let out = filters.run(Ctx::new([], &inputs), Val::from(input));

    for val in out {
      match val {
        Ok(result) => match result {
          Val::Str(s) if opts.raw_output => println!("{}", s),
          _ => {
            println!("{}", result);
          }
        },
        Err(e) => error!("Error: {}", e),
      };
    }
  } else {
    debug!("No queries successfully parsed");
  }

  Ok(())
}
