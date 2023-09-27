mod utils;

utils::test_cases!(
  unit: [
    "app.toml",
    "app-cli.toml",
    "app-http.toml",
    "app-time.toml",
    "component-composite.toml",
    "component-sql.toml",
    "component-http.toml",
    "component-wasm.toml"
  ],
  integration: []
);
