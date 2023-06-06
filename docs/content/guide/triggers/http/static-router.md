---
title: Static File Router
weight: 2
file: data/examples/http/staticfile-router.wick
kind: app_config
---

*{{< metadata field = "description" >}}*

{{% app_config_header %}}

{{% router_config_header %}}

{{<v1ref "staticrouter">}}Static Routers{{</v1ref>}} require a {{<v1ref "volume">}}Volume{{</v1ref>}} resource that serves as the root directory the router will serve files from.

{{% value path = "resources.name=DIR"  %}}

The router configuration itself takes the volume resource by the given name alongside a path to proxy from.

{{% value path = "triggers.kind=wick/trigger/http@v1.routers.volume=DIR" context = true highlight = true %}}


{{% app_config_footer %}}