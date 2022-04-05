use clap::Args;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub struct RunOptions {
  // *****************************************************************
  // Everything below is copied from common-cli-options::RunOptions
  // Flatten doesn't work with positional args...
  //
  // TODO: Eliminate the need for copy/pasting
  // *****************************************************************
  /// Name of the component to execute.
  #[clap(default_value = "default")]
  component: String,

  /// Don't read input from STDIN.
  #[clap(long = "no-input")]
  no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[clap(long = "raw", short = 'r')]
  raw: bool,

  /// Filter the outputs by port name.
  #[clap(long = "filter")]
  filter: Vec<String>,

  /// A port=value string where value is JSON to pass as input.
  #[clap(long = "data", short = 'd')]
  data: Vec<String>,

  /// Print values only and exit with an error code and string on any errors.
  #[clap(long = "values", short = 'o')]
  short: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "VINO_SEED")]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a schematic.
  #[clap(last(true))]
  args: Vec<String>,
}
