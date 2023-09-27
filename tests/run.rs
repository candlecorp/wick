mod utils;

utils::test_cases!(
  unit: [
    "anonymous-component.toml",
    "imported-component.toml",
    "stdin.toml",
    "file-reader.toml",
    "file-reader-lockdown-fail.toml",
    "file-reader-lockdown-pass.toml",
    "file-reader-lockdown-pass-wildcard-dir.toml",
    "file-reader-lockdown-pass-wildcard-components.toml",
    "wasm-command-component.toml"
  ],
  integration: [
    "postgres.toml"
  ]
);
