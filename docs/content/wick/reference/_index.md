---
title: Reference
weight: 3
---

Wick configuration is broken down in two major pieces: applications and components.

## Components

Components all follow the same basic structure, but can be configured in a variety of ways. Every component exposes operations that take inputs and produce outputs. Components and operations are analogous to modules/libraries and functions in other languages. The primary difference with `wick` is that every input and output is a stream. This lets you build components that can be composed in a variety of ways and that scale better as your applications grow.

### Component types

- [WebAssembly (WasmRS)](components/wasmrs) {{<v1ref "..."/>}}-
- [Composite](components/composite) {{<v1ref "..."/>}}-
- [SQL](components/sql) {{<v1ref "..."/>}}-
- [HTTP Client](components/http-client) {{<v1ref "..."/>}}-

## Applications

Applications translate the rest of the world into the `wick` world. They do this by defining "triggers" which operate on events, translate necessary input into wick streams, then delegate the rest of the execution to wick components.

Wick triggers:

### Application triggers

- [HTTP Trigger](triggers/http) {{<v1ref "httptrigger" />}}- A trigger whose events are HTTP requests from an HTTP server and whose operations produce HTTP responses.
- [Time Trigger](triggers/time) {{<v1ref "timetrigger" />}}- A trigger whose events are time-based and whose operations' output is logged.
- [CLI Trigger](triggers/cli) {{<v1ref "clitrigger" />}}- A trigger whose events are initiated upon CLI execution and whose operations produce exit codes.
