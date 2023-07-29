use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use wick_config::WickConfiguration;

use crate::options::reconcile_fetch_options;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  /// Path or OCI url to manifest or wasm file.
  #[clap(action)]
  path: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let xdg = wick_xdg::Settings::new();
  let bin_dir = xdg.global().root().join("bin");
  std::fs::create_dir_all(&bin_dir)?;

  span.in_scope(|| info!(app = opts.path, to = %bin_dir.display(), "installing wick app"));

  let oci_opts = reconcile_fetch_options(&opts.path, &settings, opts.oci, false, None);
  let package = crate::oci::pull(opts.path, oci_opts).await?;
  let path = package.path();
  let config = WickConfiguration::fetch(path.to_string_lossy(), Default::default())
    .await?
    .into_inner();

  let config = match config {
    WickConfiguration::App(config) => config,
    _ => anyhow::bail!("{} is not an application configuration", path.display()),
  };

  let bin_path = bin_dir.join(config.name());

  #[cfg(not(target_os = "windows"))]
  {
    let sh = format!(
      r#"
    #!/bin/sh

    wick run {}
    "#,
      path.display()
    )
    .replace("    ", "");

    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);

    std::fs::write(&bin_path, sh)?;

    std::fs::set_permissions(&bin_path, perms)?;
  }
  #[cfg(target_os = "windows")]
  {
    let mut ps1_path = bin_path.clone();
    ps1_path.set_extension("ps1");
    let mut cmd_path = bin_path.clone();
    cmd_path.set_extension("cmd");
    info!(target=%path.to_string_lossy(),cmd=%cmd_path.to_string_lossy(),ps1=%ps1_path.to_string_lossy(), "installing");
    std::fs::write(&cmd_path, make_bat(&path.to_str().unwrap()))?;
    std::fs::write(&ps1_path, make_ps1(&path.to_str().unwrap()))?;
  }

  let text = format!("installed {} to {}", config.name(), bin_path.display());
  let json = serde_json::json!({
    "name": config.name(),
    "path": bin_path,
  });

  let output = StructuredOutput::new(text, json);

  Ok(output)
}

fn make_ps1(target: &str) -> String {
  format!(
    r#"
#!/usr/bin/env pwsh
$basedir=Split-Path $MyInvocation.MyCommand.Definition -Parent

$exe=""
if ($PSVersionTable.PSVersion -lt "6.0" -or $IsWindows) {{
  # Fix case when both the Windows and Linux builds of wick
  # are installed in the same directory
  $exe=".exe"
}}
$app_path="{}"
$ret=0
if (Test-Path "$basedir\wick$exe") {{
  # Support pipeline input
  if ($MyInvocation.ExpectingInput) {{
    $input | & "$basedir\wick$exe" "run" "$app_path" $args
  }} else {{
    & "$basedir\wick$exe" "run" "$app_path" $args
  }}
  $ret=$LASTEXITCODE
}} else {{
  # Support pipeline input
  if ($MyInvocation.ExpectingInput) {{
    $input | & "wick$exe" "run" "$app_path" $args
  }} else {{
    & "wick$exe" "run" "$app_path" $args
  }}
  $ret=$LASTEXITCODE
}}
exit $ret
"#,
    target,
  )
}

fn make_bat(target: &str) -> String {
  format!(
    r#"
@ECHO off
GOTO start
:find_dp0
SET dp0=%~dp0
EXIT /b
:start
SETLOCAL
CALL :find_dp0

ECHO "%dp0%\wick.exe"

IF EXIST "%dp0%\wick.exe" (
  SET "_prog=%dp0%\wick.exe"
) ELSE (
  SET "_prog=wick"
  SET PATHEXT=%PATHEXT:;.WICK;=;%
)

endLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & "%_prog%" "run" "{}" %*
"#,
    target
  )
}
