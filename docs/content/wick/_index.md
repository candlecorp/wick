---
title: Wick Framework Documentation
weight: 0
---

# Wick Documentation

Wick is an application runtime that allows you to stitch components together into web services, CLI apps, microservices, workers, anything. Wick was designed from the ground-up with **reusability** and **security** in mind. Wick is built with rust and WebAssembly to keep your code secure and portable.

## Key Benefits of Wick

1. **Reusability**: Write a component once, use it as a microservice, JSON Rest API, CLI app, or as part of another component for larger application without changing a single line of code.

2. **Productivity**: Wick makes it easier to develop, maintain, document, and test your applications. Test every component the same way, generate documentation with a single command, and update components without fear.

3. **Security**: Wick uses WebAssembly to sandbox dependencies and keep them isolated from one another. Wick's architecture makes it easy to keep vulnerabilities _out of_ code in the first place.

### Installation

#### Install with Cargo

```
cargo install wick-cli
```

#### Install with Homebrew

```
brew install candlecorp/tap/wick
```

#### Install pre-built binaries

**Mac/Linux**

```
curl -sSL sh.wick.run | bash
```

**Windows**

```
curl https://ps.wick.run -UseBasicParsing | Invoke-Expression
```

#### Install from source

```sh
git clone https://github.com/candlecorp/wick.git && cd wick
just deps # install necessary dependencies
just install # or cargo install --path .
```

<p align="right">(<a href="#wick-documentation">back to top</a>)</p>

## Hello World!

Dive deeper into the Wick Framework by exploring the following sections:

- [Introduction](overview)
- [Getting Started](getting-started/tutorial/)
- [Core Concepts](overview/concepts)
- [Rust Component SDK Reference](rustdoc/wick_component/)
- [Examples](examples)

We're excited to have you join the Wick community! Be sure to join our public [Discord channel](https://discord.gg/candle) and share what you create!
