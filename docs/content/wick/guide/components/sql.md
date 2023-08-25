---
title: SQL Components
weight: 4
file: data/examples/db/postgres-component.wick
ref: sqlcomponent
refs:
  - name: SQL Component
    ref: sqlcomponent
  - name: SQL Operation Definition
    ref: sqloperationdefinition
  - name: Error Behavior
    ref: errorbehavior
description: A component whose operations are defined as SQL queries to a remote database.
---

SQL Components are a special type of component that allow you to define operations as SQL queries to a remote database. A SQL operation takes inputs like any other operation and it uses those inputs as bound parameters to prepared queries. The results of the query are returned as the operation's output.

## Supported Databases

- **Postgres** with connection strings that start with **`postgres://`**
- **MS SQL Server** with connection strings that start with **`mssql://`**
- **SQLite** with connection strings that start with **`sqlite://`**

## Global configuration

See the {{<v1ref "sqlcomponent">}}SQL component{{</v1ref>}} documentation for more details. Important options include:

#### `resource`

SQL Operations require a {{<v1ref "url">}}url{{</v1ref>}} resource to use as the connection string.

## Per-operation configuration

See the {{<v1ref "sqloperationdefinition">}}SQL operation definition{{</v1ref>}} documentation for more details.

#### `inputs`

The inputs are defined by the {{<v1ref "sqloperationdefinition">}}SQL operation definition{{</v1ref>}} configurations.

#### `outputs` (unconfigurable)

- **`output`** - JSON-like representation of each returned row (may be nothing).

## Example

_{{< metadata field = "description" >}}_

{{% component_config_header %}}

This example defines two operations, `get_user` and `set_user` that are backed by queries to a database.

{{% value path = "component.operations" highlight = true %}}

{{% component_config_footer %}}
