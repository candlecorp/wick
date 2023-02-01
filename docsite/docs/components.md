---
sidebar_position: 10
---

# Understanding Components

Except for Resouces, A NanoBus application configures the component types below in `bus.yaml` (or `bus.ts` if you use TypeScript configuration).

## Initializers

Components that run a task (i.e., database migrations) before your application starts receiving traffic.

## Transports

Share your application's functionality over remote protocols, such as an HTTP server or Pub/Sub subscription, so that your application is accessed by users or triggered by events.

### HTTP Middleware and Routers

An HTTP server is a barebones HTTP listener. It must have middleware and routers added to perform its intended functions. These separate components allow for the extensibility and composition of features.

## Filters

Filters are similar to HTTP middleware except for being transport agnostic. They perform tasks like parsing JWTs and reading claims before passing control to pipelines.

## Actions

Actions are where data from the call map to an operation for a specific resource, such as a database or message queue. They perform the bulk of processing in NanoBus. Actions are chained together in pipelines that make up a call to a provider (or even a no-code interface/service).

## Resources

Resources are pluggable components that allow NanoBus to interact with backend databases, message queues, or other systems. Since they plug into an application as dependencies, the system operator configures them in a separate `resourcess.yaml` file.
