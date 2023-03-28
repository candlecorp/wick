{
  "title": "JSON schema for Wick applications and components",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "$defs": {},
  "oneOf": [
    { "$ref": "#/$defs/v1/WickConfig" },
    { "$ref": "#/$defs/v0/HostManifest" }
  ]
}
