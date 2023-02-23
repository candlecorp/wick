use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use tokio::task::JoinHandle;
use wasmflow_collection_cli::options::MeshCliOptions;
use wasmflow_packet_stream::InherentData;
use wasmflow_runtime::configuration::{ApplicationContext, Channel};

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

  let app_config = wasmflow_runtime::configuration::load_yaml(&opts.location)?;
  let context = ApplicationContext {
    name: app_config.name,
    version: app_config.version,
    inherent_data,
  };

  let mut channels: Vec<Arc<Box<dyn Channel + Send + Sync>>> = vec![];
  for (name, channel_config) in app_config.channels {
    debug!("Loading channel {}", name);
    match wasmflow_runtime::configuration::get_channel_loader(&channel_config.uses) {
      Some(loader) => channels.push(Arc::new(loader(context.clone(), channel_config.with)?)),
      _ => bail!("could not find channel {}", &channel_config.uses),
    };
  }

  let mut tasks: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
  for channel in &channels {
    let task = channel.clone();
    let task = tokio::spawn(async move {
      task.run().await?;
      Ok::<(), anyhow::Error>(())
    });
    tasks.push(task);
  }

  let (item_resolved, _ready_future_index, _remaining_futures) = futures::future::select_all(tasks).await;

  for channel in channels {
    channel.shutdown_gracefully().await?;
  }

  item_resolved?
}
