mod utils;

utils::test_cases!(
  unit: [
    "jq-style.toml", "no-curlies.toml", "with-curlies.toml"
  ],
  integration: []
);
