---
title: 'Databases'
date: 2023-08-24
description: 'Leveraging database options for your Wick applications on Candle Cloud.'
weight: 5
---

# Database Management on Candle Cloud

For efficient data storage and retrieval, Candle Cloud offers integrated database solutions for your Wick applications. From local storage options to powerful databases, find the right fit for your application's data needs.

## Built-in Database Options

- **Local Storage**:

  - Every deployment on Candle Cloud comes with dedicated local storage. This storage is ideal for persistent data storage for databases or any other files.
  - Every application has access to a `/app` directory with full read-write permissions. This directory is persistent and can be used to store data that needs to be persisted across deployments.

- **SQLite**:
  - SQLite is a lightweight, file-based relational database management system. It's a great fit for applications that require a simple, efficient database without the overhead of a full-fledged RDBMS.
  - With our default entitlements, SQLite-backed applications can efficiently handle and process thousands of requests per minute, making it a robust choice for many scenarios.
  - All deployments are mounted with an environment variable `DB_URL` that points to the SQLite database file. This variable can be used in the {{<v1ref "url">}}Url{{</v1ref>}} resource to access the database file from your application.

## External Databases

While our built-in options cater to a range of applications, we understand the unique requirements of different projects:

1. **Bring Your Own Database (BYOD)**:
   - If you have specific database preferences or requirements that go beyond our built-in offerings, Candle Cloud fully supports external databases.
   - We will not impose any ingress or egress connection limits on requests to the common database ports, meaning your deployment can seamlessly connect to PostgreSQL or MSSQL databases hosted on external cloud solutions or any other platforms of your choice.

## Example Sqlite Wick Application

[Wick Examples Github](https://github.com/candlecorp/wick-apps/tree/main/sqlite-rest)

---

With the flexibility of Candle Cloud's database offerings, your Wick applications are well-equipped to handle both current and future data needs. For advanced database configurations, best practices, or troubleshooting, please visit our [Discord Channel](https://discord.gg/candle).
