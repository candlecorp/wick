# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.0 (2023-10-18)

### Chore

 - <csr-id-35ff51b8a93c27475765a7eb65c23256f4f93d67/> updated versions and changelogs
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-9bf30721df67cb244e8d82cc40f5b5f86791eb09/> updated for deprecation notice
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-11449d002b80fbc22ec5e4b684b09fbcc949a9c7/> added support for wasm component-model triggers
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

 - <csr-id-7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6/> fixed config rendering within trigger operations
 - <csr-id-a7ef87f3b06fc760a3ffe7d60da76fb343b529d2/> brought pg and sqlx type support in line with ms sql
 - <csr-id-a672dae56f4dfa4449519880093ebe4609ea6d60/> changed integration test ENV variable to avoid docker conflict
 - <csr-id-e33e83033e489ad506351c016b86a6875af10a0b/> panick on null
 - <csr-id-089e75df3fc4708d070ceb115fac6a7668737146/> fixed infinite loop on sql server queries with no inputs
 - <csr-id-0e9a4518c3136d7c6f3dbb5beec022243c2651ee/> propagating substream boundaries instead of complaining about them
 - <csr-id-9917e3a9f392712884b77f88248920c58c183c34/> surfaced errors from sql server data conversion and propagated them downstream

### Refactor

 - <csr-id-378c726823ec2fe65a168d7e205ea613b2b1c1b3/> unified input/output structs for all calls
 - <csr-id-69d79c1c8eee66dcd766648c359145a1898691c7/> removed native stdlib and associated references
 - <csr-id-0f3fef30abf88525a9966b823edccb18a1919aaf/> removed mutexes in PacketStream, made Invocation state error-proof
 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle

### New Features (BREAKING)

 - <csr-id-534d209c797d962d4fd90d590ecdb5916ecede56/> made ComponentError anyhow::Error

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 31 commits contributed to the release over the course of 113 calendar days.
 - 28 commits were understood as [conventional](https://www.conventionalcommits.org).
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
    - Unified input/output structs for all calls ([`378c726`](https://github.com/candlecorp/wick/commit/378c726823ec2fe65a168d7e205ea613b2b1c1b3))
    - Removed native stdlib and associated references ([`69d79c1`](https://github.com/candlecorp/wick/commit/69d79c1c8eee66dcd766648c359145a1898691c7))
    - Added support for wasm component-model triggers ([`11449d0`](https://github.com/candlecorp/wick/commit/11449d002b80fbc22ec5e4b684b09fbcc949a9c7))
    - Updated versions and changelogs ([`35ff51b`](https://github.com/candlecorp/wick/commit/35ff51b8a93c27475765a7eb65c23256f4f93d67))
    - Removed mutexes in PacketStream, made Invocation state error-proof ([`0f3fef3`](https://github.com/candlecorp/wick/commit/0f3fef30abf88525a9966b823edccb18a1919aaf))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`4d6e3f4`](https://github.com/candlecorp/wick/commit/4d6e3f437964552cfd6917310c17548b12e83eaf))
    - Made ComponentError anyhow::Error ([`534d209`](https://github.com/candlecorp/wick/commit/534d209c797d962d4fd90d590ecdb5916ecede56))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Fixed config rendering within trigger operations ([`7d0a399`](https://github.com/candlecorp/wick/commit/7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6))
    - Updated for deprecation notice ([`9bf3072`](https://github.com/candlecorp/wick/commit/9bf30721df67cb244e8d82cc40f5b5f86791eb09))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
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

## v0.2.0 (2023-09-14)

<csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/>
<csr-id-9bf30721df67cb244e8d82cc40f5b5f86791eb09/>
<csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/>
<csr-id-0f3fef30abf88525a9966b823edccb18a1919aaf/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-316111ac52d22365d060f573a456975de33b9115/>

### Chore

 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-9bf30721df67cb244e8d82cc40f5b5f86791eb09/> updated for deprecation notice
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

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

 - <csr-id-7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6/> fixed config rendering within trigger operations
 - <csr-id-a7ef87f3b06fc760a3ffe7d60da76fb343b529d2/> brought pg and sqlx type support in line with ms sql
 - <csr-id-a672dae56f4dfa4449519880093ebe4609ea6d60/> changed integration test ENV variable to avoid docker conflict
 - <csr-id-e33e83033e489ad506351c016b86a6875af10a0b/> panick on null
 - <csr-id-089e75df3fc4708d070ceb115fac6a7668737146/> fixed infinite loop on sql server queries with no inputs
 - <csr-id-0e9a4518c3136d7c6f3dbb5beec022243c2651ee/> propagating substream boundaries instead of complaining about them
 - <csr-id-9917e3a9f392712884b77f88248920c58c183c34/> surfaced errors from sql server data conversion and propagated them downstream

### Refactor

 - <csr-id-0f3fef30abf88525a9966b823edccb18a1919aaf/> removed mutexes in PacketStream, made Invocation state error-proof
 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle

### New Features (BREAKING)

 - <csr-id-534d209c797d962d4fd90d590ecdb5916ecede56/> made ComponentError anyhow::Error

## v0.1.0 (2023-08-28)

<csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-316111ac52d22365d060f573a456975de33b9115/>

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

