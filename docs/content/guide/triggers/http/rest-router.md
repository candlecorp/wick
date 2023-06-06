---
title: RestAPI Router
weight: 1
file: data/examples/http/rest-router.wick
kind: app_config
---

*{{< metadata field = "description" >}}*

{{% app_config_header %}}

{{% router_config_header %}}

{{<v1ref "restrouter">}}Rest Routers{{</v1ref>}} use common configuration patterns to define complex JSON (or other) RestAPIs in a simple configuration.

A {{<v1ref "restrouter">}}Rest Router{{</v1ref>}} takes any number of {{<v1ref "route">}}Routes{{</v1ref>}} that define a `uri` and `methods` to match and the {{<v1ref "componentoperationexpression">}}operation{{</v1ref>}} that will handle the request.

A {{<v1ref "route">}}Route{{</v1ref>}} uri can be defined as a simple string or can have embedded path and query string parameters which will be automatically parsed and delivered as inputs to the configured operation. E.g.

{{% value path = "triggers.kind=wick/trigger/http@v1.routers.kind=wick/router/rest@v1.routes.operation=sample::echo" context = true %}}

> Note: The request `body` will be parsed as JSON and delivered to the configured operation as the `input` parameter.

{{% app_config_footer %}}