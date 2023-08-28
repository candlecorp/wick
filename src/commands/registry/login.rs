use anyhow::Result;
use clap::Args;
use dialoguer::Password;
use structured_output::StructuredOutput;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// Registry to save credentials for.
  #[clap(action)]
  pub(crate) registry: String,

  /// Registry username. User will be prompted for password on the command line.
  #[clap(action)]
  pub(crate) username: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  mut settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let _enter = span.enter();
  let creds = settings.credentials.iter_mut().find(|c| c.scope == opts.registry);
  println!(
    "Use your registry's UI (i.e. open {} in a browser) to retrieve a token and paste it below.\n",
    opts.registry
  );

  let password = Password::new()
    .with_prompt(format!("Enter the token for {}@{}: ", opts.username, opts.registry))
    .interact()?;

  let mut json = serde_json::json!({});
  let mut lines = Vec::new();

  if let Some(creds) = creds {
    json.as_object_mut().unwrap().insert("updated".to_owned(), true.into());
    lines.push("Updating existing credentials");
    match &mut creds.auth {
      wick_settings::Auth::Basic(ref mut creds) => {
        creds.username = opts.username;
        creds.password = password;
      }
    }
  } else {
    json.as_object_mut().unwrap().insert("added".to_owned(), true.into());
    lines.push("Adding new credentials");
    settings.credentials.push(wick_settings::Credential {
      scope: opts.registry,
      auth: wick_settings::Auth::Basic(wick_settings::BasicAuth {
        username: opts.username,
        password,
      }),
    });
  }
  settings.save()?;

  Ok(StructuredOutput::new(lines.join("\n"), json))
}
