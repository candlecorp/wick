---
title: Rest APIs
---
Setup your own rest APIs
==
Wick provides an easy solution for Rest APIs as well. To set up a rest API, lets start with a `demo.wick` file. This is a standard [Wick application config]( {{< ref "configuration/reference/v1#appconfiguration" >}}) file.

To start off, like all `.wick` files, we declare the name and kind of our app/component.
```yaml
kind: wick/app@v1
name: rest_demo
```

Wick sets up Rest APIs as http triggers. In order to use this we need to declare an http [`resource`]( {{< ref "configuration/reference/v1#resourcedefinition" >}}) for our app. We can set the port that will be exposed and the address. (0.0.0.0 binds to localhost)

```yaml
resources:
  - name: http
    resource:
      kind: wick/resource/tcpport@v1
      port: 8999
      address: 0.0.0.0
```
Now that we have declared our applications needs, we can setup rest [`triggers`]( {{< ref "configuration/reference/v1#triggerdefinition" >}}).


```yaml
triggers:
  - kind: wick/trigger/http@v1
    resource: http
    routers:
      - kind: wick/router/rest@v1
        path: /api/
        routes:
```

Our trigger is setup using the http resource we provided our app above. Under [`routers`]( {{< ref "configuration/reference/v1#httprouter" >}}), we have our rest router with the ability to set its path and routes.

[`Routes`]( {{< ref "configuration/reference/v1#route" >}}) is where we get to add and define our rest apis. Let's take a look at the structure of Wick rest routes.

```yaml
routes:
 - uri: "desired url endpoint"
   operation: <component name>::<operation name>
   methods:
     - POST
```

With those 3 fields, we have set a uri destination and assigned an operation to take place when we hit that uri. We can add as many routes as desired following this same structure.

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
         - uri: "desired url endpoint"
           operation: <component name>::<operation name>
           methods:
             - POST

```

This is essentailly all you need to create rest API routes in Wick. To see how it all works, check out our http example applications [here](https://github.com/candlecorp/wick/tree/main/examples/http).
