# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.2.1 (2023-08-28)

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings

### New Features

 - <csr-id-70f0fd07ac70ae4fd1bb1734b306266f14f3af3c/> made buffer_size configurable
 - <csr-id-8fdef58ea207acb9ecb853c2c4934fe6daab39dd/> reorganized tracing span relationships
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-b679aad2e505e2e4b15794dc4decc98c51aee077/> added v1 wasm signatures, bumped wasmrs, enabled module cache
 - <csr-id-3eb6ac3742b7cebaff7cf5dbf3e552cc6cd784f3/> reused wasmtime engine from runtime, updated wasm parser
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-33c82afccdbcb4d7cda43e0ae880381501668478/> propagated seed to component context
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context

### Bug Fixes

 - <csr-id-3208691ffb824e9f83d9845ae274c9b60bb8d4fa/> converted all level spans to info_spans
 - <csr-id-e107d7cc2fb3d36925fe8af471b164c07ec3e15d/> fixed broken cache path, fixed unrendered Volume configuraton
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-5f59bb11179ee19f49c82159e3b34f3abfe1c5ab/> fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-806afef0cbc45977d782e8a1b6d79ef6ca8c397d/> removed unnecessary duplication of byte vector
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 28 commits contributed to the release over the course of 115 calendar days.
 - 131 days passed between releases.
 - 27 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#319](https://github.com/candlecorp/wick/issues/319), [#375](https://github.com/candlecorp/wick/issues/375)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **Uncategorized**
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Made buffer_size configurable ([`70f0fd0`](https://github.com/candlecorp/wick/commit/70f0fd07ac70ae4fd1bb1734b306266f14f3af3c))
    - Converted all level spans to info_spans ([`3208691`](https://github.com/candlecorp/wick/commit/3208691ffb824e9f83d9845ae274c9b60bb8d4fa))
    - Reorganized tracing span relationships ([`8fdef58`](https://github.com/candlecorp/wick/commit/8fdef58ea207acb9ecb853c2c4934fe6daab39dd))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Re-added exposing volumes to WASI components ([`ce9d202`](https://github.com/candlecorp/wick/commit/ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85))
    - Eliminated fetching of bytes before checking cache ([`586ace0`](https://github.com/candlecorp/wick/commit/586ace0978ca8adf58bf4d1fa5ed392015297c21))
    - Removed unnecessary duplication of byte vector ([`806afef`](https://github.com/candlecorp/wick/commit/806afef0cbc45977d782e8a1b6d79ef6ca8c397d))
    - Added v1 wasm signatures, bumped wasmrs, enabled module cache ([`b679aad`](https://github.com/candlecorp/wick/commit/b679aad2e505e2e4b15794dc4decc98c51aee077))
    - Reused wasmtime engine from runtime, updated wasm parser ([`3eb6ac3`](https://github.com/candlecorp/wick/commit/3eb6ac3742b7cebaff7cf5dbf3e552cc6cd784f3))
    - Fixed broken cache path, fixed unrendered Volume configuraton ([`e107d7c`](https://github.com/candlecorp/wick/commit/e107d7cc2fb3d36925fe8af471b164c07ec3e15d))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb ([`5f59bb1`](https://github.com/candlecorp/wick/commit/5f59bb11179ee19f49c82159e3b34f3abfe1c5ab))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Updated to rust 1.69.0, fixed associated warnings ([`e561fd6`](https://github.com/candlecorp/wick/commit/e561fd668afb1e1af3639c472a893b7fcfe2bf54))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Propagated seed to component context ([`33c82af`](https://github.com/candlecorp/wick/commit/33c82afccdbcb4d7cda43e0ae880381501668478))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
</details>

## v0.2.0 (2023-04-19)

### Chore

 - <csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/> release wick-cli and rest of crates
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-b83485305d609f9f599ae4a3f0aa03d9e101fb5c/> centralized relative file resolution within wick-config
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test
 - <csr-id-ade73755500573d2dec3ebf0e7113f73fa238549/> added pretty JSON output to wick invoke commands
 - <csr-id-39fb923c30ec819bcbe665ef4fad569eebdfe194/> substreams/bracketing + codegen improvements
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger

### Bug Fixes

 - <csr-id-5f346aade563554ddeb7b48c89c31dadc8ccfc5d/> fixed broken async tag for non-wasm targets

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable
 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils
 - <csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/> centralized APIs around configuration yaml

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 20 commits contributed to the release over the course of 39 calendar days.
 - 15 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Release wick-cli and rest of crates ([`1279be0`](https://github.com/candlecorp/wick/commit/1279be06f6cf8bc91641be7ab48d7941819c98fe))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
    - Centralized relative file resolution within wick-config ([`b834853`](https://github.com/candlecorp/wick/commit/b83485305d609f9f599ae4a3f0aa03d9e101fb5c))
    - Centralized APIs around configuration yaml ([`fd71df4`](https://github.com/candlecorp/wick/commit/fd71df4baaa3f856454624396eff9d9ee8c4473f))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
    - Added pretty JSON output to wick invoke commands ([`ade7375`](https://github.com/candlecorp/wick/commit/ade73755500573d2dec3ebf0e7113f73fa238549))
    - Fixed broken async tag for non-wasm targets ([`5f346aa`](https://github.com/candlecorp/wick/commit/5f346aade563554ddeb7b48c89c31dadc8ccfc5d))
    - Substreams/bracketing + codegen improvements ([`39fb923`](https://github.com/candlecorp/wick/commit/39fb923c30ec819bcbe665ef4fad569eebdfe194))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed wick-config-component to wick-config, added app config, restructured triggers, added trigger test component ([`24ef43f`](https://github.com/candlecorp/wick/commit/24ef43f7fc978c1f33f27a1e90f9971abdeb9b11))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

