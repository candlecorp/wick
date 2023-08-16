{
  "title": "Wick configuration",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "$defs": {},
  "oneOf": [
    { "$ref": "#/$defs/v1/AppConfiguration" },
    { "$ref": "#/$defs/v1/ComponentConfiguration" },
    { "$ref": "#/$defs/v1/TypesConfiguration" },
    { "$ref": "#/$defs/v1/TestConfiguration" },
    { "$ref": "#/$defs/v1/LockdownConfiguration" },
    { "$ref": "#/$defs/v0/HostManifest" }
  ]
}
