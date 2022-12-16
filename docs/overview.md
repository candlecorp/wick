# Overview

NanoBus is a lightweight application runtime that reduces developer responsibility so that teams can **focus on core logic**. It streamlines development by:

- Allowing developers to embrace the benefits of an [API-first approach](https://swagger.io/resources/articles/adopting-an-api-first-approach/), namely, having confidence that services are communicating properly and reliably at scale
- Generating boilerplate code for REST/RPC, workflow, and event-driven applications to eliminate repetitive work and minimize potential for manual errors
- Simplifying usage of cloud primitives and other microservice dependencies to basic function calls without SDKs or potentially vulerable 3rd party libraries
- Packaging your application for deployment

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

![NanoBus Architecture](./images/architecture.svg)

NanoBus runs jointly with [Dapr](https://dapr.io) in a [sidecar process](https://docs.microsoft.com/en-us/azure/architecture/patterns/sidecar). Conceptually, your application is plugged into the center and NanoBus handles bi-directional communication with Dapr and service/transport protocols.

Dapr provides developers with powerful [building blocks](https://docs.dapr.io/developing-applications/building-blocks/) such as service invocation, state management, publish and subscribe, secret stores, bindings, and actors. These building blocks are integrated with NanoBus pipelines. Pipelines are like middleware with configurable actions that perform operations like decoding, transform and routing data from the application to Dapr's components and visa-versa. No SDKs required.

To create services, NanoBus uses succinct yet flexible interface definitions to automatically produce API endpoints, like REST, [gRPC](https://grpc.io), and [NATS](https://nats.io). These transports are pluggable, allowing developers to expose services using multiple protocols without boilerplate code. Additionally, API documentation is auto-generated for customers.

Finally, NanoBus supports pluggable "compute" types: from Docker containers to emerging technologies
like [WebAssembly](https://webassembly.org). In the future, embedded language runtimes like JavaScript/TypeScript, Python or Lua could be supported.

To learn more, see the [architecture page](./architecture.md).
