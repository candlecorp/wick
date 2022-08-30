use anyhow::Result;
use clap::Args;
use tokio::task::JoinHandle;
use wasmflow_collection_cli::options::MeshCliOptions;
use wasmflow_runtime::configuration::Channel;

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
  location: String,

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

  let app_config = wasmflow_runtime::configuration::load_yaml(&opts.location)?;
  let mut channels: Vec<Box<dyn Channel + Send + Sync>> = vec![];
  for (name, channel_config) in app_config.channels.into_iter() {
    debug!("Loading channel {}", name);
    match wasmflow_runtime::configuration::get_channel_loader(&channel_config.uses) {
        Some(loader) => channels.push(loader(channel_config.with)?),
        _ => bail!("could not find channel {}", &channel_config.uses),
    };
  }

  let mut tasks: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
  for channel in channels.into_iter() {
    let task = tokio::spawn(async move {
      channel.run().await?;
      // what to do after?
      Ok::<(), anyhow::Error>(()) 
    });
    tasks.push(task);
  }

  let (item_resolved, _ready_future_index, _remaining_futures) =
    futures::future::select_all(tasks).await;

  // TODO: Figure out borrow issue here.
  // for channel in channels.into_iter() {
  //   channel.shutdown_gracefully().await;
  // }

  item_resolved?

  // Ok(())
}
