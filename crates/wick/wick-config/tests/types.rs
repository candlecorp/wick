mod utils;
mod integration_test {

  use anyhow::Result;

  use crate::utils::load;

  #[test_logger::test(tokio::test)]
  async fn test_external_types() -> Result<()> {
    let config = load("./tests/manifests/v1/import-types.yaml").await?;
    let component = config.try_component_config()?;
    let signature = component.signature()?;
    let type_names = signature.types.iter().map(|t| t.name().to_owned()).collect::<Vec<_>>();
    assert_eq!(
      type_names,
      vec![
        "http::HttpMethod".to_owned(),
        "http::HttpScheme".to_owned(),
        "http::HttpVersion".to_owned(),
        "http::StatusCode".to_owned(),
        "http::HttpResponse".to_owned(),
        "http::HttpRequest".to_owned(),
      ]
    );

    Ok(())
  }
}
