// use std::borrow::Cow;

// use wasmflow_manifest::process_default;

// pub(crate) fn make_default_transport(json: &serde_json::Value, message: &str) -> Result<Vec<u8>, String> {
//   process_default(Cow::Borrowed(json), message).map_or(Err("Error processing default value".to_owned()), |result| {
//     wasmrs_codec::messagepack::serialize(&result)
//       .map_or(Err("Error serializing default value".to_owned()), |bytes| Ok(bytes))
//   })
// }
