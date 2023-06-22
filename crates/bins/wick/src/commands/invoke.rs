use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use seeded_random::Seed;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_component_cli::options::DefaultCliOptions;
use wick_component_cli::parse_args;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_packet::{InherentData, Packet, PacketStream};

use crate::options::get_auth_for_scope;
use crate::utils::{self, merge_config, parse_config_string};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  /// Turn on info logging.
  #[clap(long = "info", action)]
  pub(crate) info: bool,

  /// Path or OCI url to manifest or wasm file.
  #[clap(action)]
  path: String,

  /// Name of the operation to execute.
  #[clap(default_value = "default", action)]
  operation: String,

  /// Don't read input from STDIN.
  #[clap(long = "no-input", action)]
  no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[clap(long = "raw", short = 'r', action)]
  raw: bool,

  /// Filter the outputs by port name.
  #[clap(long = "filter", action)]
  filter: Vec<String>,

  /// Print values only and exit with an error code and string on any errors.
  #[clap(long = "values", action)]
  short: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// Pass configuration necessary to instantiate the component (JSON).
  #[clap(long = "with", short = 'w', action)]
  with: Option<String>,

  /// Pass configuration necessary to invoke the operation (JSON).
  #[clap(long = "op-with", action)]
  op_with: Option<String>,

  /// Arguments to pass as inputs to a component.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let configured_creds = settings.credentials.iter().find(|c| opts.path.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci.username.as_deref(),
    opts.oci.password.as_deref(),
  );
  let env = wick_xdg::Settings::new();

  let mut fetch_opts: wick_oci_utils::OciOptions = opts.oci.clone().into();
  fetch_opts.set_username(username).set_password(password);

  let path = PathBuf::from(&opts.path);

  if !path.exists() {
    fetch_opts.set_cache_dir(env.global().cache().clone());
  } else {
    let mut path_dir = path.clone();
    path_dir.pop();
    fetch_opts.set_cache_dir(path_dir.join(env.local().cache()));
  };

  let manifest = WickConfiguration::fetch_all(&opts.path, fetch_opts)
    .await?
    .try_component_config()?;

  let server_options = DefaultCliOptions { ..Default::default() };

  let mut manifest = merge_config(&manifest, &opts.oci, Some(server_options));

  let root_config = parse_config_string(opts.with.as_deref())?;
  let op_config = parse_config_string(opts.op_with.as_deref())?;
  manifest.set_root_config(root_config);
  manifest.initialize(None)?;

  let component = opts.operation;

  let mut host = ComponentHostBuilder::default().manifest(manifest).span(span).build()?;

  host.start_runtime(opts.seed.map(Seed::unsafe_new)).await?;

  let signature = host.get_signature()?;
  let op_signature = signature
    .get_operation(&component)
    .ok_or_else(|| anyhow::anyhow!("operation not found: {}", component))?;

  let check_stdin = if op_signature.inputs.is_empty() {
    false
  } else {
    !opts.no_input && opts.args.is_empty()
  };

  let inherent_data = opts.seed.map_or_else(InherentData::unsafe_default, |seed| {
    InherentData::new(
      seed,
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    )
  });

  if check_stdin {
    todo!("STDIN support is not yet implemented.");
    // if atty::is(atty::Stream::Stdin) {
    //   eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    // }
    // let reader = io::BufReader::new(io::stdin());
    // let mut lines = reader.lines();

    // while let Some(line) = lines.next_line().await? {
    //   debug!("STDIN:'{}'", line);
    //   let mut payload = TransportMap::from_json_output(&line)?;
    //   if !opts.raw {
    //     payload.transpose_output_name();
    //   }

    //   let stream = host.request(&default_schematic, payload, inherent_data).await?;

    //   utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
    // }
  } else {
    let args = parse_args(&opts.args, op_signature)?;
    trace!(args= ?args, "parsed CLI arguments");
    let mut packets = Vec::new();
    let mut seen_ports = HashSet::new();
    for packet in args {
      seen_ports.insert(packet.port().to_owned());
      packets.push(Ok(packet));
    }
    for port in seen_ports {
      packets.push(Ok(Packet::done(port)));
    }
    debug!(args= ?packets, "invoke");
    let stream = PacketStream::new(futures::stream::iter(packets));

    let stream = host.request(&component, op_config, stream, inherent_data).await?;

    utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }
  host.stop().await;

  Ok(StructuredOutput::new("", json!({})))
}
