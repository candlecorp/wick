# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.5.1 (2023-08-28)

### Chore

 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases

### New Features

 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-33c82afccdbcb4d7cda43e0ae880381501668478/> propagated seed to component context
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-302612d5322fcc211b1ab7a05969c6de4bca7d7e/> added switch/case operation
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component

### Bug Fixes

 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

### Refactor

 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation
 - <csr-id-ff8b81dc1be6ff70237aaea1bc501b623f7c14d1/> merged PacketStream into Invocation for invocations

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 22 commits contributed to the release over the course of 123 calendar days.
 - 131 days passed between releases.
 - 21 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#319](https://github.com/candlecorp/wick/issues/319), [#375](https://github.com/candlecorp/wick/issues/375)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **Uncategorized**
    - Support provides/requires relationship in composite components ([`8ceae1a`](https://github.com/candlecorp/wick/commit/8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax ([`b0b9cd2`](https://github.com/candlecorp/wick/commit/b0b9cd20f748ffe1956ad2501fe23991fededf13))
    - Added inheritance/delegation to composite components, reorganized test files ([`e46db5f`](https://github.com/candlecorp/wick/commit/e46db5f2138254c227a2c39a3821074b77cf0166))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Merged PacketStream into Invocation for invocations ([`ff8b81d`](https://github.com/candlecorp/wick/commit/ff8b81dc1be6ff70237aaea1bc501b623f7c14d1))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Propagated seed to component context ([`33c82af`](https://github.com/candlecorp/wick/commit/33c82afccdbcb4d7cda43e0ae880381501668478))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added switch/case operation ([`302612d`](https://github.com/candlecorp/wick/commit/302612d5322fcc211b1ab7a05969c6de4bca7d7e))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
</details>

## v0.5.0 (2023-04-18)

### Chore

 - <csr-id-e9e2d75ca15477fb910e1780e650d695c842168e/> removed unused dev-dependencies in flow-component
 - <csr-id-7361b149ca108904341364426e1509105913f31f/> release
   flow-component, flow-expression-parser, flow-graph, wick-asset-reference, wick-component, wick-config, wick-oci-utils
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 1 calendar day.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Removed unused dev-dependencies in flow-component ([`e9e2d75`](https://github.com/candlecorp/wick/commit/e9e2d75ca15477fb910e1780e650d695c842168e))
    - Release ([`7361b14`](https://github.com/candlecorp/wick/commit/7361b149ca108904341364426e1509105913f31f))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
</details>

