---
title: Core Concepts
---
Wick is designed to enable developers to "compose" applications using a configuration file and invoke components based on the needs of the application. This section covers the core concepts behind Wick's composition model and its main benefits.

### Application Composition

In Wick, applications are composed by defining components and their relationships in a configuration file. This configuration-driven approach simplifies the development process by abstracting the underlying complexity and focusing on the high-level structure of the application.

### Components
Components are the building blocks of a Wick application. They are modular, reusable pieces of code that can be combined and orchestrated to create complex applications. Components in Wick are WebAssembly modules that can be invoked based on the requirements of the application.

### Configuration (wick.yaml)
The configuration file (YAML) is where developers define the components and their relationships. It specifies which components to use, how they should be connected, and any necessary configurations for the components to function properly. The configuration-driven approach enables developers to easily modify and extend applications by simply updating the configuration file.

### Component Invocation
Based on the application's needs, Wick components can be invoked. Components can have one or more "operations" (functions), which represent the different actions that a component can perform.

### Resources
In Wick, components don't have direct access to the network, filesystems, environment or system variables. Instead, they can express their "needs" for external resources. The Wick application configuration is responsible for ensuring that the needs of all components are met by defining the resources and permissions available to each component. Examples of resources include HTTP clients and database client drivers (MySQL, MSSQL, etc.).

### Triggers

Triggers in Wick are external elements that can invoke a Wick application. They serve as the entry point for a Wick application and can take various forms, such as:

* HTTP Server: Triggers the application based on incoming HTTP requests.
* GRPC Server: Triggers the application based on incoming GRPC calls.
* Time-Based Scheduler: Triggers the application at specific time intervals or scheduled times.
* CLI: Triggers the application based on command-line input.

By defining and using triggers, developers can create applications that respond to different types of input and events, allowing for more versatile and adaptable applications.