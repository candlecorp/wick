pub(crate) fn pull_into_dir(url: String, dir: String) -> anyhow::Result<()> {
  let args = cargo_generate::Args {
    allow_commands: false,
    list_favorites: false,
    favorite: Some(url),
    subfolder: None,
    git: None,
    path: None,
    branch: None,
    name: Some(dir),
    force: false,
    verbose: false,
    template_values_file: None,
    silent: false,
    config: None,
    vcs: cargo_generate::Vcs::Git,
    lib: false,
    bin: false,
    ssh_identity: None,
    define: vec![],
    init: false,
    force_git_init: false,
  };

  cargo_generate::generate(args)
}
