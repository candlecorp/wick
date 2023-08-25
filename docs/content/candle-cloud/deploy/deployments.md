---
title: 'Deployments'
date: 2023-08-24
description: 'Step-by-step guide to deploying your Wick applications on Candle Cloud.'
weight: 1
---

# Deploying Wick Applications on Candle Cloud

Deploying your Wick applications on Candle Cloud is a streamlined process designed to get your applications up and running in no time. This guide walks you through the deployment process, ensuring you leverage the robust capabilities of Candle Cloud effectively.

## Starting a New Deployment

1. **Initiate Deployment**: Navigate to the `Deployments` section on your Candle Cloud dashboard. Click on the `+` icon located in the top right corner.

2. **Name Your Deployment**:

   - Provide a name for your deployment.
   - Note: They must be in lowercase and can contain hyphens (a-z, 1-9, and -) (e.g., `my-wick-app`).

3. **Registry Reference**:

   - Input the registry reference where your Wick package is stored.
   - Currently, Candle Cloud only supports packages hosted on `registry.candle.dev`.
   - e.g., `registry.candle.dev/my-namespace/my-app:0.1.0`

4. **Configure Application Port**:

   - Specify the port your application listens on.
   - This should align with the {{<v1ref "tcpport">}}TcpPort{{</v1ref>}} resource configuration set within your Wick application.

5. **Wick Binary Version**:

   - Choose between the `latest` or `nightly` version of the Wick binary based on your preference and application requirements.
     - `latest`: This is the most recent stable release of Wick.
     - `nightly`: This represents the cutting-edge version with the newest features. However, it might not be as stable as the `latest`.

6. **Finalize & Deploy**: After filling in the necessary details, review your configurations. Click on the deploy button to initiate the process. Once deployed, your Wick application will start running based on the specified configurations.

## Deployment Host Name

Your application will be available on the internet. Once it is deployed, you can access it using the following URL format:

```bash
https://<app-name>.<tenant-name>.apps.wick.run
```

There will be a link to the URL on your deployment details section.

---

With your application deployed, you can monitor its health directly from the Candle Cloud dashboard. If you have any questions or need assistance, please visit our [Discord Channel](https://discord.gg/candle).
