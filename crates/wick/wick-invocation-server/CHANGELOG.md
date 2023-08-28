# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.1 (2023-08-28)

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context

### Bug Fixes

 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature

### Refactor

 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 115 calendar days.
 - 131 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#375](https://github.com/candlecorp/wick/issues/375)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **Uncategorized**
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
</details>

## v0.1.0 (2023-04-19)

### Chore

 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger

### Test

 - <csr-id-890b9dd879e9d18c8e989989a01e73eb5a987b2f/> moved tests with native-component to separate rust project in test/integration
 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 39 calendar days.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Moved tests with native-component to separate rust project in test/integration ([`890b9dd`](https://github.com/candlecorp/wick/commit/890b9dd879e9d18c8e989989a01e73eb5a987b2f))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

