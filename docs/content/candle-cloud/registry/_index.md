---
title: Registry Service
weight: 1
description: 'Candle Cloud Registry service.'
---

# Pushing to Candle Cloud Registry

Candle Cloud Registry makes it easy to store and manage your Wick packages. Follow the steps below to push your package:

## 1. Logging In

1. Visit [registry.candle.dev](https://registry.candle.dev).
2. Click on the `Login` button.
3. Choose your preferred login method:
   - Google
   - GitHub
   - Microsoft
   - Or, create an account using a username and password specifically for Candle Cloud.

## 2. Setting Up Your Namespace

1. If it is your first time logging in, you'll be prompted to create a username. This ensures a unique identity within the Candle Cloud ecosystem.
2. Navigate to `My Projects`.
3. Click on `New Project`.
4. Enter a ` project name` and choose if you prefer it to be `public` or `private`. We encourage everyone to try `public` for any components that you want to share with the community, but understand the need for `private` apps as well.
5. The `project name` will serve as your `namespace` in the [Wick package](../../wick/getting-started/package) configuration.

## 3. Getting your Wick CLI Registry Credentials

1. Click on your username in the top right corner.
2. Select `Profile`
3. Copy the `Secret` and `Username` values.
4. Set the credentials using the options avaiable in the [Wick package](../../wick/getting-started/package#registry-credentials).

## 4. Pushing Your Package

To push your package, you can use the `wick registry push` command. For example:

```bash
wick registry push <app/component.wick>
```

For more information on Wick Packages, please refer to the [Wick package](../../wick/getting-started/package) documentation. For any further assistance, please visit our [Discord Channel](https://discord.gg/candle).
