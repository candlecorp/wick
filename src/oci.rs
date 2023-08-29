pub(crate) async fn pull(
  reference: String,
  oci_opts: wick_oci_utils::OciOptions,
) -> Result<wick_package::WickPackage, anyhow::Error> {
  let pull_result = match wick_package::WickPackage::pull(&reference, &oci_opts).await {
    Ok(pull_result) => pull_result,
    Err(e) => {
      if let wick_package::Error::Oci(wick_oci_utils::error::OciError::WouldOverwrite(files)) = &e {
        warn!("pulling {} will overwrite the following files", &reference);
        for file in files {
          warn!("{}", file.display());
        }
        error!("refusing to overwrite files, pass --force to ignore.");
        return Err(anyhow!("Pull failed"));
      }
      error!("failed to pull {}: {}", &reference, e);
      return Err(anyhow!("Pull failed"));
    }
  };
  Ok(pull_result)
}
