use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use seeded_random::Seed;
use wick_component_cli::options::DefaultCliOptions;
use wick_component_cli::parse_args;
use wick_config::{FetchOptions, WickConfiguration};
use wick_host::ComponentHostBuilder;
use wick_packet::{InherentData, Packet, PacketStream};

use crate::options::get_auth_for_scope;
use crate::utils::{self, merge_config, parse_config_string};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct InvokeCommand {
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

  /// A port=value string where value is JSON to pass as input.
  #[clap(long = "data", short = 'd', action)]
  data: Vec<String>,

  /// Print values only and exit with an error code and string on any errors.
  #[clap(long = "values", short = 'o', action)]
  short: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// Pass configuration necessary to instantiate the component (JSON).
  #[clap(long = "with", short = 'w', action)]
  with: Option<String>,

  /// Arguments to pass as inputs to a component.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle(opts: InvokeCommand, settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
  let configured_creds = settings.credentials.iter().find(|c| opts.path.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci.username.as_deref(),
    opts.oci.password.as_deref(),
  );

  let mut fetch_opts = FetchOptions::default()
    .allow_insecure(opts.oci.insecure_registries.clone())
    .allow_latest(true);
  if let Some(username) = username {
    fetch_opts = fetch_opts.oci_username(username);
  }
  if let Some(password) = password {
    fetch_opts = fetch_opts.oci_password(password);
  }

  if !PathBuf::from(&opts.path).exists() {
    fetch_opts = fetch_opts.artifact_dir(wick_xdg::Directories::GlobalCache.basedir()?);
  };

  let manifest = WickConfiguration::fetch_all(&opts.path, fetch_opts)
    .await?
    .try_component_config()?;

  let server_options = DefaultCliOptions { ..Default::default() };

  let config = merge_config(&manifest, &opts.oci, Some(server_options));

  let component = opts.operation;

  let component_config = parse_config_string(opts.with.as_deref())?;

  let mut host = ComponentHostBuilder::default()
    .manifest(config)
    .config(component_config)
    .span(span)
    .build()?;

  host.start_engine(opts.seed.map(Seed::unsafe_new)).await?;

  let signature = host.get_signature()?;
  let target_schematic = signature.get_operation(&component);

  let mut check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();
  if let Some(target_schematic) = target_schematic {
    if target_schematic.inputs.is_empty() {
      check_stdin = false;
    }
  }

  let inherent_data = opts.seed.map(|seed| {
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
    let data = Packet::from_kv_json(&opts.data)?;

    let args = parse_args(&opts.args)?;
    trace!(args= ?args, "parsed CLI arguments");
    let mut packets = Vec::new();
    let mut seen_ports = HashSet::new();
    for packet in args {
      seen_ports.insert(packet.port().to_owned());
      packets.push(Ok(packet));
    }
    for packet in data {
      seen_ports.insert(packet.port().to_owned());
      packets.push(Ok(packet));
    }
    for port in seen_ports {
      packets.push(Ok(Packet::done(port)));
    }
    debug!(args= ?packets, "invoke");
    let stream = PacketStream::new(futures::stream::iter(packets));

    let stream = host.request(&component, stream, inherent_data).await?;
    utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }
  host.stop().await;

  Ok(())
}
