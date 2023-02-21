pub(crate) fn pull_into_dir(url: &str, dir: String) -> anyhow::Result<()> {
  git2::Repository::clone(url, dir)?;
  Ok(())
}
