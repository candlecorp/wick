---
title: Introduction
---

# What is Wick?
Wick is a software development framework that aims to revolutionize the way developers create valuable applications by addressing the unique challenges faced by application developers. By leveraging WebAssembly, a standardized bytecode, Wick provides an innovative platform that is secure, composable, and highly maintainable.

## Philosophy
Wick acknowledges the differences between application developers and library developers. While both types of developers create immense value, they face distinct challenges and often use the same tools designed for library developers. Wick is specifically designed as a platform for application developers, enabling them to build scalable and adaptable applications with ease.

## Overview
At a high level, Wick orchestrates code and manages communication between dependencies. It can work with various types of code, such as WebAssembly modules, external microservices, or anything else that speaks the Wick protocol.

Wick combines functional and flow-based concepts from projects like Erlang, Rx, FBP, and Haskell, along with ideas from Docker and Kubernetes, to create a powerful platform for application development.

## Key Benefits
Wick offers numerous advantages to developers, including:

1. Security: Wick uses WebAssembly to sandbox all dependencies and restricts functionality to CPU only by default. It isolates memory per transaction, providing a secure environment for applications. With Wick, you can safely run third-party precompiled code without the risk of data exfiltration, ensuring the security and integrity of your application's data.
2. Composability: Developers can compile code into Wick components and connect them together with a manifest, which can then be used as a collection of new components. This enables a high degree of code reusability.
3. Productivity: Wick normalizes the interfaces in and out of WebAssembly, simplifying complex integrations and allowing developers to swap dependencies with minimal effort.
4. Testability: Wick's test runner enables developers to run unit tests rapidly on the command line without the need for complex integrations.
5. Maintainability: Wick allows for seamless scaling and modification of applications, as any part can be turned into a microservice, worker, or other component without rebuilding.
6. Versatility: Wick allows you to run the same code on both server and client sides, reducing development time and effort while maintaining consistency across your application.

## What Can Wick Build?
Wick is a versatile and opinionated runtime designed to support a wide range of application types. With Wick, developers can effortlessly build web servers, microservices, background workers, command-line applications, or client-side applications using the same framework. Built on the principles of security, reusability, and composability, Wick aims to revolutionize the application development landscape, providing a robust and flexible platform for developers to create valuable software across various domains.
