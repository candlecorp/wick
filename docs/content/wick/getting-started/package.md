---
title: 'Package'
date: 2023-08-24
description: 'Guide to creating and pushing Wick packages.'
weight: 10
---

Packaging your Wick application or component is a key step before deployment. This guide provides a comprehensive walkthrough of creating a Wick package and pushing it to an OCI compliant registry.

## Wick Package Configuration

Here's a typical configuration block for creating a package with Wick:

```yaml
---
kind: wick/app@v1
name: my-app
metadata:
  version: 0.1.0
 ...
package:
  registry:
    host: registry.candle.dev
    namespace: wick
  files:
    - ./public
```

## Configuration Details:

- **host**: Specifies the OCI compliant registry where your Wick package will be pushed. We highly recommend using the free [Candle Registry](https://registry.candle.dev), which is optimized specifically for Wick packages.
- **namespace**: Represents the "project" or container under which the package artifact will reside. If you're familiar with DockerHub, this is similar to a repository.
- **files**: An array that lists the directories or files to be included in the package. In this example, all files in the ./public directory will be added to the package. This is useful for including static assets such as html, images, CSS, and JavaScript files for web applications and SPA.

## Packaging Name and Version

Wick automatically extracts the package name and version from the Wick application (or component) manifest. This ensures consistency and eliminates manual errors during the packaging process.

## Registry Credentials

There are two ways to set your registry credentials:
First is to use environment variables:

```bash
export OCI_USERNAME=<username>
export OCI_PASSWORD=<password/secret>
```

The second way is to set the values in the wick `config.yaml` file located at `~/.wick/config.yaml`:

```yaml
---
 ...
credentials: #this is a top level key
  - scope: registry.candle.dev
    auth:
      type: basic
      username: <username>
      password: <password/secret>
 ...
```

## Pushing a Package

Once your configuration is set and you're ready to push your package, use the following command:

```bash
wick registry push <app/component.wick> --tag=latest
```

This command pushes your Wick application or component (specified by <app/component.wick>) to the registry, tagging it as the latest version.

## Running a Package

Once the package is pushed, you can run it anywhere using the following command:

```bash
wick run registry.candle.dev/wick/my-app:0.1.0
```

## Package Types

All wick apps, components, and types can be indepenently packaged and pushed to the registry. This allows for easy sharing and reusability of any and all parts of a wick application or component.

## Import Types

A common use case for packaging types is to import them into other wick applications or components. This allows you to share types across multiple applications and components, ensuring consistency and reducing the need for duplicate code. You may frequently use the `http` type. Here is an example of using the `http` type in a wick application:

```yaml
---
name: 'my-component'
kind: wick/component@v1
metadata:
  version: '0.0.1'
  ...
import:
  - name: http
    component:
      kind: wick/component/types@v1
      ref: registry.candle.dev/types/http:0.4.0 # <-- import the http type
  ...
component:
  kind: wick/component/wasmrs@v1
  ref: build/component.signed.wasm
  operations:
    - name: my_operation
      inputs:
        - name: request
          type: http::HttpRequest # <-- using the imported type
```

## Contents of a Package

- The package will contain any `*.wick` files that are directly referenced by your parent file.
- Additionally, the package contains a tar of all the external files that are referenced in your `package.files` configuration.
- Finally, the package will contain your compiled WebAssembly binary if you are using the `wick/component/wasmrs@v1` component. Your source code is not included in the package by default and we recommend that you do not include it. If you wish to make your source code public, you can add it to any public git repository and reference the link to it in your app/component metadata section.

## Examples

You will find many examples of package and component configurations in the [Wick Components](https://github.com/candlecorp/wick-components/tree/main/components) repository.
