---
title: Rest APIs
---
Setup your own rest APIs
==
Wick provides an easy solution for Rest APIs as well. To set up a rest API, lets start with a `demo.wick` file. This is a standard Wick application config file.

To start off, like all `.wick` files, we declare the name and kind of our app/component.
```yaml
kind: wick/app@v1
name: rest_demo
```

Wick sets up Rest APIs as http triggers. In order to use this we need to declare an http resource for our app. We can set the port that will be exposed and the address. (0.0.0.0 binds to localhost)

```yaml
resources:
  - name: http
    resource:
      kind: wick/resource/tcpport@v1
      port: 8999
      address: 0.0.0.0
```
Now that we have declared our applications needs, we can setup rest `triggers`.


```yaml
triggers:
  - kind: wick/trigger/http@v1
    resource: http
    routers:
      - kind: wick/router/rest@v1
        path: /api/
        routes:
```

Our trigger is setup using the http resource we provided our app above. Under `routers`, we have our rest router with the ability to set its path and routes.

`Routes` is where we get to add and define our rest apis. Let's take a look at the structure of Wick rest routes.

```yaml
routes:
 - uri: "desired url endpoint"
   operation: <component name>::<operation name>
   methods:
     - POST
```

With those 3 fields, we have set a uri destination and assigned an operation to take place when we hit that uri. We can add as many routes as desired following this same structure.

### Done!

This is essentailly all you need to create rest API routes in Wick. To see how it all works, check out our full web application demo.