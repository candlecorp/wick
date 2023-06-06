---
title: Proxy Router
weight: 3
file: data/examples/http/proxy-router.wick
kind: app_config
---

*{{< metadata field = "description" >}}*

{{% app_config_header %}}

{{% router_config_header %}}

{{<v1ref "proxyrouter">}}Proxy Routers{{</v1ref>}} require a {{<v1ref "url">}}Url{{</v1ref>}} resource that serves as the URL the proxy router will forward requests to.

{{% value path = "resources.name=PROXY_URL" highlight = true %}}

The router configuration itself takes the url resource by the given name alongside a path to proxy from.

> Note: The `path` can be stripped from the proxied path by specifying `strip_path: true`.


{{% value path = "triggers.kind=wick/trigger/http@v1.routers.url=PROXY_URL" context = true highlight = true %}}


{{% app_config_footer %}}