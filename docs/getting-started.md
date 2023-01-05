# Getting Started

## Prerequisites

- [Apex CLI][apex]: A project template and code generation tool.
- [just]: A task runner similar to Make.

## Install Nanobus

Use the instructions at [github.com/nanobus/nanobus][nanobus] to install pre-compiled releases for your architecture.

If you're building from source, clone this repository and run `just install` to install `nanobus` to your local go path.

```sh
$ git clone git@github.com:nanobus/nanobus.git && cd nanobus
$ just install
```

## Hello World!

Use the `apex` CLI to start a new project from the starter template:

```console
$ apex new git@github.com:nanobus/nanobus.git -p templates/init my-app
```

The starter template includes up a nanobus configuration with one sample interface and an action that takes an input named `name`.

```yaml
id: "my-app"
version: 0.0.1
interfaces:
  Greeter:
    SayHello:
      steps:
        - name: Say Hello!
          uses: expr
          with:
            value: '"Hello, " + input.name'
```

Run our action with the `nanobus invoke` command with sample input piped from `echo`:

```console
$ echo '{"name": "World!"}' | nanobus invoke Greeter SayHello
"Hello, World!"
```

The starter template also includes a tiny `justfile` with this command set up already.

```console
$ just run
echo '{"name": "World!"}' | nanobus invoke Greeter SayHello
"Hello, World!"
```

## Making a web service

To expose our interface to something other than the command line, we use a `transport`. Transports are ways of attaching interfaces to event-based resources, like a web servers, message queues, or schedulers. There are several transports baked into nanobus.

To spin up an HTTP server, use the transport `nanobus.transport.http.server/v1`. Use the configuration below to configure it to use port `8080` and use the internal router `nanobus.transport.http.router/v1`

```yaml
transports:
  http:
    uses: nanobus.transport.http.server/v1
    with:
      address: ":8080"
      routers:
        - uses: nanobus.transport.http.router/v1
          with:
            routes:
              - method: POST
                uri: /hello
                handler: Greeter::SayHello
```

Each route is configured with its method, uri, handler and optional encoding. By default, the encoding is set to the content-type or `application/json` but you can specify other formats with the `encoding` property.

Our full configuration now looks like this:

```yaml
id: "my-app"
version: 0.0.1
transports:
  http:
    uses: nanobus.transport.http.server/v1
    with:
      address: ":8080"
      routers:
        - uses: nanobus.transport.http.router/v1
          with:
            routes:
              - method: POST
                uri: /hello
                handler: Greeter::SayHello
interfaces:
  Greeter:
    SayHello:
      steps:
        - name: Say Hello!
          uses: expr
          with:
            value: '"Hello, " + input.name'
```

Start our web service by running `nanobus` and watch nanobus initialize our HTTP server and routes automatically.

```console
$ nanobus
INFO	Initializing transport	{"name": "http"}
INFO	Serving route	{"uri": "/hello", "methods": "POST", "handler": "Greeter::SayHello"}
INFO	HTTP server listening	{"address": ":8080"}
```

Make a request with a tool like curl to see the output:

```sh
$ curl 127.0.0.1:8080/hello -H "Content-Type: application/json" \
  -d '{"name": "World!"}'
{"type":"permission_denied","code":"permission_denied","status":403,"path":"/hello","timestamp":"2022-12-21T17:26:56.067483Z"}
```

Permission denied!

An important part of nanobus is making it difficult to cut corners that can lead to catastrophes later. It's too easy to build APIs that ignore authentication and authorization. Adding it as a layer on top of an existing application leads to security holes and major vulnerabilities. Every action in nanobus is deny-by-default. You must explicitly configure what is public and – if it's private – what permissions and roles can access what actions. We haven't run into this error yet because `nanobus invoke` on the command line bypasses configured auth requirements.

Add an `authorization` block to our configuration and set up our action as unauthenticated.

```yaml
authorization:
  Greeter:
    SayHello:
      unauthenticated: true
```

Our full web service configuration now looks like this:

```yaml
id: "my-app"
version: 0.0.1
transports:
  http:
    uses: nanobus.transport.http.server/v1
    with:
      address: ":8080"
      routers:
        - uses: nanobus.transport.http.router/v1
          with:
            routes:
              - method: POST
                uri: /hello
                handler: Greeter::SayHello
authorization:
  Greeter:
    SayHello:
      unauthenticated: true
interfaces:
  Greeter:
    SayHello:
      steps:
        - name: Say Hello!
          uses: expr
          with:
            value: '"Hello, " + input.name'
```

Run it with the command `nanobus` and issue a curl command to see our output.


```sh
$ curl 127.0.0.1:8080/hello -H "Content-Type: application/json" -d '{"name": "World!"}'
"Hello, World!"
```

Congrats! You just put together a web service with nothing more than yaml. We've barely scratched the surface. Extending nanobus with more serious logic is where the power really takes off.


[apex]: https://apexlang.io
[just]: https://github.com/casey/just#packages
[nanobus]: https://github.com/nanobus/nanobus