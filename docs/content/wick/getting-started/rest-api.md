---
title: RestAPI
weight: 2
---

# Set up your own RestAPI

Wick provides an easy solution for RestAPIs as well. To set one up, let's start with a `demo.wick` file. This is a standard {{<v1ref "appconfiguration">}}Wick application config{{</v1ref>}} file.

To start off, like all `.wick` files, we declare the name and kind of our app/component.

```yaml
kind: wick/app@v1
name: rest_demo
```

Wick sets up Rest APIs as HTTP triggers. In order to use this, we need to declare an HTTP {{<v1ref "resourcedefinition">}}resource{{</v1ref>}} for our app. We can set the port that will be exposed and the address. (0.0.0.0 binds to localhost)

```yaml
resources:
  - name: http
    resource:
      kind: wick/resource/tcpport@v1
      port: 8999
      address: 0.0.0.0
```

Now that we have declared our application's needs, we can set up REST {{<v1ref "triggerdefinition">}}triggers{{</v1ref>}}.

```yaml
triggers:
  - kind: wick/trigger/http@v1
    resource: http
    routers:
      - kind: wick/router/rest@v1
        path: /api/
        routes:
```

Our trigger is set up using the HTTP resource we provided to our app above. Under {{<v1ref "httprouter">}}routers{{</v1ref>}}, we have our REST router with the ability to set its path and routes.

{{<v1ref "route">}}Routes{{</v1ref>}} is where we get to add and define our REST APIs. Let's take a look at the structure of Wick REST routes.

```yaml
routes:
  - uri: 'desired URL endpoint'
    operation: <component name>::<operation name>
    methods:
      - POST
```

With those three fields, we have set a URI destination and assigned an operation to take place when we hit that URI. We can add as many routes as desired following this same structure.

### Done!

Here is what your complete `demo.wick` file would look like:

```yaml
kind: wick/app@v1
name: rest_demo
resources:
  - name: http
    resource:
      kind: wick/resource/tcpport@v1
      port: 8999
      address: 0.0.0.0
triggers:
  - kind: wick/trigger/http@v1
    resource: http
    routers:
      - kind: wick/router/rest@v1
        path: /api/
        routes:
          - uri: 'desired URL endpoint'
            operation: <component name>::<operation name>
            methods:
              - POST
```

This is essentially all you need to create REST API routes in Wick. To see how it all works, check out our HTTP example applications [here](https://github.com/candlecorp/wick/tree/main/examples/http).
