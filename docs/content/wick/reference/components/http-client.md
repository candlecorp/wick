---
title: HTTP Client
weight: 3
file: data/examples/components/http-client.wick
ref: httpclientcomponent
description: |
  A component whose operations are defined as HTTP requests to a remote server.
refs:
  - name: HTTP Client Component
    ref: httpclientcomponent
  - name: HTTP Client Operation Definition
    ref: httpclientoperationdefinition
  - name: HTTP Method
    ref: httpmethod
  - name: Codec
    ref: codec
  - name: Liquid JSON
    ref: liquidjsonvalue
description: A component whose operations are defined as SQL queries to a remote database.
---

HTTP Components are a special type of component that allow you to define operations whose implementation is backed by an HTTP request. An HTTP operation takes inputs like any other operation and uses those inputs to build up URL paths, query parameters, and request body data.

## Global configuration

See the {{<v1ref "httpclientcomponent">}}HTTP Client component{{</v1ref>}} documentation for more details. Important options include:

#### `resource`

HTTP Client Operations require a {{<v1ref "url">}}url{{</v1ref>}} resource to use as the base url for operation requests.

#### `codec`

The codec defines how the request and response bodies should be encoded and decoded. See the {{<v1ref "codec">}}codec{{</v1ref>}} documentation for allowed values.

## Per-operation configuration

See the {{<v1ref "httpclientoperationdefinition">}}HTTP Client operation definition{{</v1ref>}} documentation for more details.

#### `inputs`

The inputs to HTTP Client operations are defined by the {{<v1ref "httpclientoperationdefinition">}}HTTP Client operation definition{{</v1ref>}} configurations.

#### `outputs` (unconfigurable)

- **`response`** - The response object (status, headers, et al) returned by the HTTP request.
- **`body`** - Bytes or JSON dependending on the {{<v1ref "codec">}}codec{{</v1ref>}}


## Example

*{{< metadata field = "description" >}}*

{{% component_config_header %}}

This example defines one operation, `httpbin_get`, that uses HTTPBIN to echo our query parameters back to us. Notice how we use Liquid syntax in our path to dynamically build up the URL.

{{% value path = "component.operations" highlight = true %}}

{{% component_config_footer %}}