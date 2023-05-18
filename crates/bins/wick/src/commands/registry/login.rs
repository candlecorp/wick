use anyhow::Result;
use clap::Args;
use dialoguer::Password;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RegistryLoginCommand {
  /// Registry to save credentials for.
  #[clap(action)]
  pub(crate) registry: String,

  /// Registry username. User will be prompted for password on the command line.
  #[clap(action)]
  pub(crate) username: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: RegistryLoginCommand,
  mut settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<()> {
  let _enter = span.enter();
  let creds = settings.credentials.iter_mut().find(|c| c.scope == opts.registry);
  println!(
    "Use your registry's UI (i.e. open {} in a browser) to retrieve a token and paste it below.\n",
    opts.registry
  );

  let password = Password::new()
    .with_prompt(format!("Enter the token for {}@{}: ", opts.username, opts.registry))
    .interact()?;

  if let Some(creds) = creds {
    debug!("Updating existing credentials");
    match &mut creds.auth {
      wick_settings::Auth::Basic(ref mut creds) => {
        creds.username = opts.username;
        creds.password = password;
      }
    }
  } else {
    debug!("Adding new credentials");
    settings.credentials.push(wick_settings::Credential {
      scope: opts.registry,
      auth: wick_settings::Auth::Basic(wick_settings::BasicAuth {
        username: opts.username,
        password,
      }),
    });
  }
  settings.save()?;

  Ok(())
}
