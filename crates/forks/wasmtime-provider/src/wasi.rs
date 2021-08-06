use std::error::Error;
use std::ffi::OsStr;
use std::path::{
  Component,
  PathBuf,
};

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use wasi_common::WasiCtx;

pub(crate) fn init_ctx(
  preopen_dirs: &[(String, Dir)],
  argv: &[String],
  env: &[(String, String)],
) -> Result<WasiCtx, Box<dyn Error>> {
  let mut ctx_builder = wasi_cap_std_sync::WasiCtxBuilder::new();

  ctx_builder = ctx_builder.inherit_stdio().args(argv)?.envs(env)?;

  for (name, file) in preopen_dirs {
    ctx_builder = ctx_builder.preopened_dir(file.try_clone()?, name)?;
  }

  Ok(ctx_builder.build())
}

pub(crate) fn compute_preopen_dirs(
  dirs: &[String],
  map_dirs: &[(String, String)],
) -> Result<Vec<(String, Dir)>, Box<dyn Error>> {
  let mut preopen_dirs = Vec::new();

  for dir in dirs.iter() {
    preopen_dirs.push((
      dir.clone(),
      Dir::open_ambient_dir(dir, ambient_authority())?,
    ));
  }

  for (guest, host) in map_dirs.iter() {
    preopen_dirs.push((
      guest.clone(),
      Dir::open_ambient_dir(host, ambient_authority())?,
    ));
  }

  Ok(preopen_dirs)
}

#[allow(dead_code)]
pub(crate) fn compute_argv(module: PathBuf, module_args: Vec<String>) -> Vec<String> {
  // Add argv[0], which is the program name. Only include the base name of the
  // main wasm module, to avoid leaking path information.
  let mut result = vec![module
    .components()
    .next_back()
    .map(Component::as_os_str)
    .and_then(OsStr::to_str)
    .unwrap_or("")
    .to_owned()];

  // Add the remaining arguments.
  for arg in module_args.iter() {
    result.push(arg.clone());
  }

  result
}
