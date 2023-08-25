---
title: Raw Router
weight: 4
file: data/examples/http/raw-router.wick
kind: app_config
---

*{{< metadata field = "description" >}}*

{{% app_config_header %}}

{{% router_config_header %}}

{{<v1ref "rawrouter">}}Raw Routers{{</v1ref>}} delegate an HTTP request directly to a wick component. In addition to the router `path` configuration, raw routers require a reference to an {{<v1ref "componentoperationexpression">}}operation{{</v1ref>}} that will handle the request.

{{% value path = "triggers.kind=wick/trigger/http@v1.routers.kind=wick/router/raw@v1" context = true highlight = true %}}

Operations can be defined inline or referenced by name from a previously imported component. The `operation` value above comes from a previously imported `import` block:

{{% value path = "import" context = false highlight = true %}}





{{% app_config_footer %}}