---
title: 'Secrets'
date: 2023-08-24
description: 'Guide to securely handling and deploying secrets in your Wick applications on Candle Cloud.'
weight: 2
---

# Secrets Management on Candle Cloud

Securing sensitive data is paramount in today's software landscape. With Candle Cloud's secrets management feature, you can securely handle confidential data, ensuring they're shielded from prying eyes but accessible to your applications when needed.

## Creating a Secret

1. **Access Secrets Section**: Navigate to the `Secrets` section on your Candle Cloud dashboard.

2. **Initiate Secret Creation**: Click on the `+` icon, found in the top right corner or as part of the secrets section.

3. **Name Your Secret**:

   - Assign a name to your secret.
   - Naming Requirement: They must be in uppercase and can contain underscores (A-Z, 1-9, and \_) (e.g., `DATABASE_PASSWORD`).

4. **Input Secret Value**:

   - Enter the corresponding value for the secret.
   - For security reasons, once saved, the secret's actual value will not be visible within the Candle Cloud dashboard.

5. **Save & Secure**: After defining the secret name and its value, save the secret. It will be securely stored and encrypted on Candle Cloud.

## Updating Secrets

While you can't view the actual value of a stored secret, updating it is straightforward:

1. Navigate to the desired secret within the `Secrets` section.
2. Click on the update or edit button, usually represented as a pencil icon or similar.
3. Modify the secret's value and save the changes.

## Accessing Secrets in Deployments

All the secrets you create on Candle Cloud are seamlessly integrated into your deployments. They are injected as environment variables, available to your Wick applications. This ensures a secure handoff of sensitive data without hardcoding or exposing them in your application code.

For instance, if you have a secret named `DATABASE_PASSWORD`, it will be accessible in your application as an environment variable with the same name. In Wick applications, you can access the environment variables by using the `{{ ctx.env.<variable-name> }}` syntax.

---

Leverage the power of secrets on Candle Cloud allows you to freely share your applications without worrying about secrets being exposed. If you have any questions or need assistance, please visit our [Discord Channel](https://discord.gg/candle).
