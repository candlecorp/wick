use std::path::PathBuf;

use wasmtime_wasi::preview2::{DirPerms, FilePerms, Table, WasiCtx};
use wasmtime_wasi::{ambient_authority, Dir};

use crate::Error;

pub(crate) fn init_ctx(
  table: &mut Table,
  preopen_dirs: Vec<(String, Dir)>,
  argv: &[String],
  env: &[(String, String)],
) -> Result<WasiCtx, Error> {
  let mut ctx_builder = wasmtime_wasi::preview2::WasiCtxBuilder::new();

  ctx_builder.inherit_stdio().args(argv).envs(env);

  for (path, opendir) in preopen_dirs {
    let dir_perms = DirPerms::all();
    let file_perms = FilePerms::all();
    ctx_builder.preopened_dir(opendir, dir_perms, file_perms, path);
  }

  ctx_builder.build(table).map_err(Error::WasiCtx)
}

pub(crate) fn compute_preopen_dirs<'a, T: Iterator<Item = (&'a String, &'a PathBuf)>>(
  map_dirs: T,
) -> Result<Vec<(String, Dir)>, Error> {
  let ambient_authority = ambient_authority();
  let mut preopen_dirs = Vec::new();

  for (guest_path, host_path) in map_dirs {
    preopen_dirs.push((
      guest_path.clone(),
      Dir::open_ambient_dir(host_path, ambient_authority).map_err(Error::OpenDir)?,
    ));
  }

  Ok(preopen_dirs)
}
