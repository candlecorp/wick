---
sidebar_position: 2
title: Overview
---

# Overview <small>(3 min read)</small>

## Why did we build NanoBus?

We built NanoBus to let people develop the apps of their dreams without toil.

We've built enough software to see that 95% of our applications are built the same. We wanted a platform that gives us the 95% for free, so we can focus on what we're passionate about, building great solutions for users.

## What is NanoBus?

NanoBus is a lightweight framework for building secure and scalable software services.

## Who should consider NanoBus?

NanoBus has something for everyone.

**For Developers and Architects:**

- NanoBus uses strict contracts to constrain the scope of projects. Contract-first development improves predictability and integration across teams.

- NanoBus uses those contracts to generate code and automates documentation, support configuration, and schemas.

- NanoBus standardizes the protocol between dependencies and applications, so scaling an application from a single process to microservices in the cloud is just a configuration change.

**For Testers:**

- NanoBus modules (Iotas) are strictly defined and easy to test without additional integration tools. They don't rely on massive application state, and every Iota is testable the same way. A service composed of a dozen iotas is tested the same way as one single dependency.

**For Security Teams:**

- NanoBus moves critical functions like resource access and authentication from code to external configuration, making audits quick and easy.

- NanoBus isolates dependencies in secure sandboxes to eliminate their ability to cause damage even if exploited.

**For DevOps and Platform Teams:**

- Iotas standardize the unit of distribution for code, just like docker containers did for processes. Build, deploy, audit, and run everything the same way.

- Every NanoBus application comes pre-wired with all the tools that make applications easy to manage and maintain.

**For Product Owners:**

- NanoBus follows API-driven development. Applications start from the outward-facing touch points and documentation first and let everyone be on the same page before a line of code is written.

## The Origin of NanoBus

We've been building software since before the internet. We've worked at large and small companies in every industry, from finance to health care and security to video games.

We're all building things, solving problems, and eventually arriving at identical solutions.

Despite the similarity between every application and company, we're still writing and rewriting custom code to meet needs.

Every company builds software that aligns with how they communicate and organize. They draw boundaries and assign milestones that keep teams working independently. Each company operates differently, resulting in software filled with vast amounts of custom code that offer no differentiation.

We need modern tools that understand how we organize and accommodate the needs of all teams and roles, not just developers.

## Key Features

### Virtually no boilerplate code

NanoBus allows the developer to focus on what matters most, the application's logic. All the distributed system "glue" is handled automatically.

### Data pipelines

Communicating with other services and cloud primitives/building blocks is simplified in declarative, composable pipelines. Secure API endpoints, transform data, support multiple serialization formats, and apply resiliency policies using succinct configuration.

### Automatic API endpoints with documentation

Share your service through multiple protocols, including REST, gRPC, and NATS, without writing additional code. Provide your partner teams with OpenAPI/Swagger UI, AsyncAPI, and Protobuf documentation.

### Consistent polyglot programming model

Using NanoBus and Dapr as a sidecar greatly simplifies distributed application development. The developer experience feels like local development with plain interfaces and data structures, regardless of the chosen programming language.

### Clean Architecture

NanoBus applications are structured with design principles that allow your application to scale as requirements evolve. Newly created projects use an intuitive layout that follows best practices like [separation of concerns](https://en.wikipedia.org/wiki/Separation_of_concerns).

## How It Works

![NanoBus Architecture](/img/architecture.svg)

Conceptually, your application is plugged into the center, and NanoBus handles bi-directional communication via composable pipelines. Pipelines are like middleware with configurable actions that perform decoding, transforming, and routing data. No SDKs are required.

Optionally, pipelines can integrate with [Dapr](https://dapr.io), providing developers with powerful [building blocks](https://docs.dapr.io/developing-applications/building-blocks/) such as service invocation, state management, publish and subscribe, secret stores, bindings, and actors. These building blocks are integrated with NanoBus pipelines.

To create services, NanoBus uses succinct yet flexible interface definitions to automatically produce API endpoints, like REST, [gRPC](https://grpc.io), and [NATS](https://nats.io). These transports are pluggable, allowing developers to expose services using multiple protocols without boilerplate code. Additionally, API documentation is auto-generated for customers.

Finally, NanoBus runs code compiled to [WebAssembly](https://webassembly.org). WebAssembly components run in a secure sandbox and insulate core logic from external resources and dependencies.

See the [architecture page](./architecture.mdx).
