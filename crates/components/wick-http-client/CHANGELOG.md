# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0 (2023-08-28)

### Chore

 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases

### New Features

 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-dd57e5062f3cf5d01e163ad104e56f7debc50aa4/> added xml codec for wick-http-component
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-1528f18c896c16ba798d37dcca5e017beecfd7c2/> added openapi spec generation
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-bd050c4c9bc32a5bee432045088cafcc5c13e7c3/> updated liquid-json to enable more complex templates
 - <csr-id-e2abceed2d1cc7436fbe4631d3eac861ae91675e/> updated headers to be liquidjson
 - <csr-id-85abe5adc703a9190b82dd78f58acdfe9920e3fe/> added better packet output in debugging mode
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-5495686f598e766a73c240554e5c8fbdfb297376/> added form-data codec to http client
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-ba94e4dd43a85bb0dd79953f92b5a053e1536e62/> added op config to http client operations, added builders for config types
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/> add Base64Bytes to wick-packet
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component

### Bug Fixes

 - <csr-id-090d3f65564cb70a2cf1cee0c4fb4c4001e11d36/> added additional trace for http debugging
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-7af24ca9477e4d224682e170ae3a561ec237d181/> parse recusive body as json
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-9053e403a32eff847be6d43e623a464fa0377395/> fixed sql bound arguments and postgres encodings

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-be57f85e388c38265c33d457339c4dbf5f1ae65f/> renamed XML codec to Text
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle
 - <csr-id-b590d66e24d1e1dd582656b54b896586e9c8f4fb/> adjusted data types, fixed code-genned files
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation
 - <csr-id-ff8b81dc1be6ff70237aaea1bc501b623f7c14d1/> merged PacketStream into Invocation for invocations

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 38 commits contributed to the release over the course of 123 calendar days.
 - 36 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 6 unique issues were worked on: [#232](https://github.com/candlecorp/wick/issues/232), [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#341](https://github.com/candlecorp/wick/issues/341), [#347](https://github.com/candlecorp/wick/issues/347), [#399](https://github.com/candlecorp/wick/issues/399)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#341](https://github.com/candlecorp/wick/issues/341)**
    - Added ctx.inherent.timestamp, improved error message output ([`efe6055`](https://github.com/candlecorp/wick/commit/efe605510b846d2556f6060ba710fa154bdca7c4))
 * **[#347](https://github.com/candlecorp/wick/issues/347)**
    - Added `core::collect` component to collect a stream into a single object ([`2e6462a`](https://github.com/candlecorp/wick/commit/2e6462a8574ca5a09e0522ec7ff42ca4429657ba))
 * **[#399](https://github.com/candlecorp/wick/issues/399)**
    - Better http client substream support. ([`744f1ac`](https://github.com/candlecorp/wick/commit/744f1ac3d5fa8c28e8e0a1e80d7f5e49839c0c43))
 * **Uncategorized**
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Support provides/requires relationship in composite components ([`8ceae1a`](https://github.com/candlecorp/wick/commit/8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d))
    - Renamed XML codec to Text ([`be57f85`](https://github.com/candlecorp/wick/commit/be57f85e388c38265c33d457339c4dbf5f1ae65f))
    - Added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax ([`b0b9cd2`](https://github.com/candlecorp/wick/commit/b0b9cd20f748ffe1956ad2501fe23991fededf13))
    - Re-added exposing volumes to WASI components ([`ce9d202`](https://github.com/candlecorp/wick/commit/ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85))
    - Added xml codec for wick-http-component ([`dd57e50`](https://github.com/candlecorp/wick/commit/dd57e5062f3cf5d01e163ad104e56f7debc50aa4))
    - Adjusted logging, interpreter execution lifecycle ([`316111a`](https://github.com/candlecorp/wick/commit/316111ac52d22365d060f573a456975de33b9115))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Added openapi spec generation ([`1528f18`](https://github.com/candlecorp/wick/commit/1528f18c896c16ba798d37dcca5e017beecfd7c2))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Added additional trace for http debugging ([`090d3f6`](https://github.com/candlecorp/wick/commit/090d3f65564cb70a2cf1cee0c4fb4c4001e11d36))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Updated liquid-json to enable more complex templates ([`bd050c4`](https://github.com/candlecorp/wick/commit/bd050c4c9bc32a5bee432045088cafcc5c13e7c3))
    - Adjusted data types, fixed code-genned files ([`b590d66`](https://github.com/candlecorp/wick/commit/b590d66e24d1e1dd582656b54b896586e9c8f4fb))
    - Parse recusive body as json ([`7af24ca`](https://github.com/candlecorp/wick/commit/7af24ca9477e4d224682e170ae3a561ec237d181))
    - Updated headers to be liquidjson ([`e2abcee`](https://github.com/candlecorp/wick/commit/e2abceed2d1cc7436fbe4631d3eac861ae91675e))
    - Added better packet output in debugging mode ([`85abe5a`](https://github.com/candlecorp/wick/commit/85abe5adc703a9190b82dd78f58acdfe9920e3fe))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Merged PacketStream into Invocation for invocations ([`ff8b81d`](https://github.com/candlecorp/wick/commit/ff8b81dc1be6ff70237aaea1bc501b623f7c14d1))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Fixed sql bound arguments and postgres encodings ([`9053e40`](https://github.com/candlecorp/wick/commit/9053e403a32eff847be6d43e623a464fa0377395))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Added form-data codec to http client ([`5495686`](https://github.com/candlecorp/wick/commit/5495686f598e766a73c240554e5c8fbdfb297376))
    - Added op config to http client operations, added builders for config types ([`ba94e4d`](https://github.com/candlecorp/wick/commit/ba94e4dd43a85bb0dd79953f92b5a053e1536e62))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Add Base64Bytes to wick-packet ([`399c5d5`](https://github.com/candlecorp/wick/commit/399c5d518b0a291dba63fb3f69337af2911d1776))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
</details>

