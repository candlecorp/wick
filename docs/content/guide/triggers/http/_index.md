---
title: HTTP Trigger
weight: 1
file: data/examples/http/proxy-router.wick
description: A trigger whose events are HTTP requests from an HTTP server and whose operations produce HTTP responses.
ref: httptrigger
---

Wick's {{<v1ref "httptrigger">}}HTTP Trigger{{</v1ref>}} initializes an HTTP server and routes incoming requests to configured {{<v1ref "router">}}routers{{</v1ref>}}.

The {{<v1ref "httptrigger">}}HTTP Trigger{{</v1ref>}} takes a {{<v1ref "tcpport">}}TCP Port{{</v1ref>}} resource and a list of {{<v1ref "router">}}routers{{</v1ref>}} that act on requests.

## Routers

- [Rest Router](http/rest-router) {{<v1ref "restrouter"/>}}- A router that turns Wick component's into a JSON Rest API.
- [Static Router](http/static-router) {{<v1ref "staticrouter"/>}}- A router that serves static files from a configured volume or directory.
- [Proxy Router](http/proxy-router) {{<v1ref "proxyrouter"/>}}- A router that proxies requests to a configured URL.
- [Raw Router](http/raw-router) {{<v1ref "rawrouter"/>}}- A router that delegates requests directly to a configured operation.
