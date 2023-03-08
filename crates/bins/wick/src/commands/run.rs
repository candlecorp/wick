use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use tokio::task::JoinHandle;
use wick_component_cli::options::MeshCliOptions;
use wick_config::AppConfiguration;
use wick_packet::InherentData;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  pub(crate) mesh: MeshCliOptions,

  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  /// The path or OCI URL to a wafl manifest or wasm file.
  #[clap(action)]
  path: String,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WAFL_SEED", action)]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a component.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  let _guard = logger::init(&opts.logging.name(crate::BIN_NAME));

  debug!(args = ?opts.args, "rest args");

  let _inherent_data = opts.seed.map(|seed| {
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

  let app_config = AppConfiguration::load_from_file(&opts.path)?;

  let mut tasks: Vec<JoinHandle<Result<(), wick_runtime::error::RuntimeError>>> = vec![];
  for channel_config in app_config.triggers() {
    debug!("Loading channel {:?}", channel_config);
    let config = channel_config.clone();
    let name = app_config.name().to_owned();

    match wick_runtime::get_trigger_loader(&channel_config.kind()) {
      Some(loader) => tasks.push(tokio::spawn(async move { loader()?.run(name, config).await })),
      _ => bail!("could not find channel {:?}", &channel_config),
    };
  }

  let (item_resolved, _ready_future_index, _remaining_futures) = futures::future::select_all(tasks).await;

  Ok(item_resolved??)
}
