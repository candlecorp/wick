# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0 (2023-08-28)

### Chore

 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors

### New Features

 - <csr-id-4516bb7034d4dbe0ffbe6625df32302d40e63570/> support volume restrictions on file:// urls, in-mem SQLite DBs
 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-6d1949b2bc1012e9314b6e2e0637ac2225c87614/> improved type coercion, added mssql tests
 - <csr-id-71ba0230aadd9c31d05ebef3478247dbf200fa1d/> brought postgres and sqlite up-to-date with mssql
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-1832a3243bb89c85bf357aea53dddce5da218bdd/> added optionals support to azure sql impl
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.

### Bug Fixes

 - <csr-id-a7ef87f3b06fc760a3ffe7d60da76fb343b529d2/> brought pg and sqlx type support in line with ms sql
 - <csr-id-a672dae56f4dfa4449519880093ebe4609ea6d60/> changed integration test ENV variable to avoid docker conflict
 - <csr-id-e33e83033e489ad506351c016b86a6875af10a0b/> panick on null
 - <csr-id-089e75df3fc4708d070ceb115fac6a7668737146/> fixed infinite loop on sql server queries with no inputs
 - <csr-id-0e9a4518c3136d7c6f3dbb5beec022243c2651ee/> propagating substream boundaries instead of complaining about them
 - <csr-id-9917e3a9f392712884b77f88248920c58c183c34/> surfaced errors from sql server data conversion and propagated them downstream

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 19 commits contributed to the release over the course of 63 calendar days.
 - 18 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#328](https://github.com/candlecorp/wick/issues/328), [#341](https://github.com/candlecorp/wick/issues/341), [#345](https://github.com/candlecorp/wick/issues/345)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#341](https://github.com/candlecorp/wick/issues/341)**
    - Added ctx.inherent.timestamp, improved error message output ([`efe6055`](https://github.com/candlecorp/wick/commit/efe605510b846d2556f6060ba710fa154bdca7c4))
 * **[#345](https://github.com/candlecorp/wick/issues/345)**
    - Added `exec`-style SQL operation ([`1162c1d`](https://github.com/candlecorp/wick/commit/1162c1d4bef87d585d76be7bb4b55811aa946796))
 * **Uncategorized**
    - Support volume restrictions on file:// urls, in-mem SQLite DBs ([`4516bb7`](https://github.com/candlecorp/wick/commit/4516bb7034d4dbe0ffbe6625df32302d40e63570))
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Brought pg and sqlx type support in line with ms sql ([`a7ef87f`](https://github.com/candlecorp/wick/commit/a7ef87f3b06fc760a3ffe7d60da76fb343b529d2))
    - Support provides/requires relationship in composite components ([`8ceae1a`](https://github.com/candlecorp/wick/commit/8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d))
    - Improved type coercion, added mssql tests ([`6d1949b`](https://github.com/candlecorp/wick/commit/6d1949b2bc1012e9314b6e2e0637ac2225c87614))
    - Brought postgres and sqlite up-to-date with mssql ([`71ba023`](https://github.com/candlecorp/wick/commit/71ba0230aadd9c31d05ebef3478247dbf200fa1d))
    - Added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax ([`b0b9cd2`](https://github.com/candlecorp/wick/commit/b0b9cd20f748ffe1956ad2501fe23991fededf13))
    - Re-added exposing volumes to WASI components ([`ce9d202`](https://github.com/candlecorp/wick/commit/ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85))
    - Changed integration test ENV variable to avoid docker conflict ([`a672dae`](https://github.com/candlecorp/wick/commit/a672dae56f4dfa4449519880093ebe4609ea6d60))
    - Panick on null ([`e33e830`](https://github.com/candlecorp/wick/commit/e33e83033e489ad506351c016b86a6875af10a0b))
    - Fixed infinite loop on sql server queries with no inputs ([`089e75d`](https://github.com/candlecorp/wick/commit/089e75df3fc4708d070ceb115fac6a7668737146))
    - Propagating substream boundaries instead of complaining about them ([`0e9a451`](https://github.com/candlecorp/wick/commit/0e9a4518c3136d7c6f3dbb5beec022243c2651ee))
    - Adjusted logging, interpreter execution lifecycle ([`316111a`](https://github.com/candlecorp/wick/commit/316111ac52d22365d060f573a456975de33b9115))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Surfaced errors from sql server data conversion and propagated them downstream ([`9917e3a`](https://github.com/candlecorp/wick/commit/9917e3a9f392712884b77f88248920c58c183c34))
    - Added optionals support to azure sql impl ([`1832a32`](https://github.com/candlecorp/wick/commit/1832a3243bb89c85bf357aea53dddce5da218bdd))
</details>

