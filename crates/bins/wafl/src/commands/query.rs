use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use jaq_core::{parse, Ctx, Definitions, Val};
use markup_converter::Transcoder;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// Option to print raw output.
  #[clap(short = 'r', long = "raw", action)]
  raw_output: bool,
  
  /// Path to JSON, YAML, or TOML file.
  #[clap(action)]
  path: PathBuf,

  /// The query.
  #[clap(action)]
  query: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let input = Transcoder::new(&opts.path)?.to_json()?;

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

    // iterator over the output values
    let out = filters.run(Ctx::new(), Val::from(input));

    for val in out {
      match val {
        Ok(result) => match result{
            Val::Str(s) if opts.raw_output => println!("{}", s),
            _ => {
                println!("{}", result)
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
