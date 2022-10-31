![NanoBus Logo](docs/images/nanobus-logo.svg)

NanoBus is a lightweight microservice runtime that reduces developer responsibility so that teams can **focus on core application logic**. It streamlines development by:

* Allowing developers to embrace the benefits of an [API-first approach](https://swagger.io/resources/articles/adopting-an-api-first-approach/), namely, having confidence that services are communicating properly and reliably at scale
* Generating boilerplate code for REST/RPC, workflow, and event-driven applications to eliminate repetitive work and minimize potential for manual errors
* Simplifying usage of cloud primitives and other microservice dependencies to basic function calls without SDKs or potentially vulerable 3rd party libraries
* Packaging your application for deployment

<!--
NanoBus is a lightweight microservice runtime layer that simplifies your application's core logic by moving infrastructure concerns to composable pipelines. The primary goal of NanoBus is to codify best practices so developers can **focus on business outcomes, not boilerplate code**.
-->

## Key Features

### Virtually no boilerplate code

In conjunction with Dapr, NanoBus allows the developer to focus on what matters most, the application's logic. All the distributed system "glue" is handled automatically.

### Data pipelines

Communicating with other services and cloud primitives/building blocks is simplified in declarative, composable pipelines. Secure API endpoints, transform data, support multiple serialization formats, and apply resiliency policies using succinct configuration.

### Automatic API endpoints with documentation

Share your service through multiple protocols, including REST, gRPC and NATS, without writing additional code. Provide OpenAPI/Swagger UI, AsyncAPI and Protobuf documentation to your partner teams.

### Consistent polyglot programming model

Using NanoBus and Dapr as a sidecar greatly simplifies distributed application development. Regardless of the chosen programming language, the developer experience feels like local development with plain interfaces and data structures.

### Clean Architecture

NanoBus applications are structured with design principles that allow your application to scale as requirements evolve. Newly created projects use an intuitive layout that follow best practices like [separation of concerns](https://en.wikipedia.org/wiki/Separation_of_concerns).

## How It Works

![NanoBus Architecture](docs/images/architecture.svg)

NanoBus runs jointly with [Dapr](https://dapr.io) in a [sidecar process](https://docs.microsoft.com/en-us/azure/architecture/patterns/sidecar). Conceptually, your application is plugged into the center and NanoBus handles bi-directional communication with Dapr and service/transport protocols.

Dapr provides developers with powerful [building blocks](https://docs.dapr.io/developing-applications/building-blocks/) such as service invocation, state management, publish and subscribe, secret stores, bindings, and actors. These building blocks are integrated with NanoBus pipelines. Pipelines are like middleware with configurable actions that perform operations like decoding, transform and routing data from the application to Dapr's components and visa-versa. No SDKs required.

To create services, NanoBus uses succinct yet flexible interface definitions to automatically produce API endpoints, like REST, [gRPC](https://grpc.io), and [NATS](https://nats.io). These transports are pluggable, allowing developers to expose services using multiple protocols without boilerplate code. Additionally, API documentation is auto-generated for customers.

Finally, NanoBus supports pluggable "compute" types: from Docker containers to emerging technologies
like [WebAssembly](https://webassembly.org). In the future, embedded language runtimes like JavaScript/TypeScript, Python or Lua could be supported.

To learn more, see the [architecture page](/docs/architecture.md).

## Getting Started

### Install the Apex for Code Generation

NanoBus are generated using [Apex](https://apexlang.io). The [documentation](https://apexlang.io/docs/getting-started#cli) goes over installation and offers a tutorial.

Next, from a terminal, install NanoBus code generators for Apex.

To generate projects, use [`just`](https://github.com/casey/just#packages), an alternative to `make`, to build.


```cli
git clone https://github.com/nanobus/iota.git
cd iota/codegen
just install
just apex-install
```

Note: The above step will be simplified once these packages are published to [NPM](https://www.npmjs.com).

Now you'll want to install the NanoBus runtime (this repo). You can build from source or [install a release](./install/README.md).

```cli
git clone https://github.com/nanobus/nanobus.git
cd nanobus
make install
```



### Create a NanoBus Application

Choose a currently supported language:

* WASM TinyGo (@nanobus/tinygo)
* WASM Rust (@nanobus/rust)

Coming soon...

* Node.js
* C# / .NET
* Python
* Golang
* AssemblyScript (WASM)
* Java (Reactor)
* Rust (Native)

```shell
apex new @nanobus/tinygo hello_world
cd hello_world
just
nanobus
```

In NanoBus, the developer only needs to follow these steps:

1. Create a new service using `apex new [template] [directory]` or `apex init [template]`
2. Define the services interfaces by editing `apex.aidl`
3. Define pipelines in `bus.yaml` that tie operations to infrastructure building blocks
4. Implement the service's core logic code
5. Run `just build`
6. Deploy to your favorite container orchestrator

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on the code of conduct and the process for submitting pull requests.

## License

This project is licensed under the [Elastic License 2.0](https://www.elastic.co/licensing/elastic-license) - see the [LICENSE.txt](LICENSE.txt) file for details
