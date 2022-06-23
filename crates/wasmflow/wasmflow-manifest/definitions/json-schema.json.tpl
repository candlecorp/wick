{
  "title": "JSON schema for Wasmflow host manifests",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "$defs": {},
  "oneOf": [
    { "$ref": "#/$defs/v1/WasmflowManifest" },
    { "$ref": "#/$defs/v0/HostManifest" }
  ]
}
