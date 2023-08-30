# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.1 (2023-08-28)

### Chore

 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### New Features

 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-33c82afccdbcb4d7cda43e0ae880381501668478/> propagated seed to component context
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context

### Bug Fixes

 - <csr-id-978690e2d05b2cae05991d273876a28f845abbb5/> fixed check in tests that resulted in duplicate DONE packets being sent
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing

### Refactor

 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 115 calendar days.
 - 131 days passed between releases.
 - 11 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#319](https://github.com/candlecorp/wick/issues/319), [#375](https://github.com/candlecorp/wick/issues/375)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **Uncategorized**
    - Fixed check in tests that resulted in duplicate DONE packets being sent ([`978690e`](https://github.com/candlecorp/wick/commit/978690e2d05b2cae05991d273876a28f845abbb5))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Propagated seed to component context ([`33c82af`](https://github.com/candlecorp/wick/commit/33c82afccdbcb4d7cda43e0ae880381501668478))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
</details>

## v0.1.0 (2023-04-19)

### Chore

 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al

### New Features

 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Refactor (BREAKING)

 - <csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/> removed "default" value substitution in favor of a future impl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 14 commits contributed to the release over the course of 39 calendar days.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Removed "default" value substitution in favor of a future impl ([`c7b84da`](https://github.com/candlecorp/wick/commit/c7b84daacad21d9ba2c44123a6b0695db3b43528))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Incorporated interface.json into component.yaml ([`97280ee`](https://github.com/candlecorp/wick/commit/97280ee71b361472dbb6ae32c77626b07c218554))
    - Shoring up tests. fixed error propagation and hung txs stemming from timeouts ([`46310b9`](https://github.com/candlecorp/wick/commit/46310b98b6933c5a6d84c32863391bb482af5ac3))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

