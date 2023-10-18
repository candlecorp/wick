# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.0 (2023-10-18)

<csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/>
<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-599514816356f7fab3b2122156092166f7815427/>
<csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/>
<csr-id-0f3fef30abf88525a9966b823edccb18a1919aaf/>
<csr-id-550524ba42ba6b302173d6c27982f01a75b4f41d/>
<csr-id-43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-806afef0cbc45977d782e8a1b6d79ef6ca8c397d/>
<csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/>

### Chore

 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings

### Refactor

 - <csr-id-69d79c1c8eee66dcd766648c359145a1898691c7/> removed native stdlib and associated references

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-2ce019fed2c7d9348c9c47d5221d322e700ce293/> added support for wasm imports
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

 - <csr-id-0f3fef30abf88525a9966b823edccb18a1919aaf/> removed mutexes in PacketStream, made Invocation state error-proof
 - <csr-id-550524ba42ba6b302173d6c27982f01a75b4f41d/> removed mutex from context config
 - <csr-id-43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5/> lowercased the start character of all log events
 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-806afef0cbc45977d782e8a1b6d79ef6ca8c397d/> removed unnecessary duplication of byte vector
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release over the course of 5 calendar days.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Removed native stdlib and associated references ([`69d79c1`](https://github.com/candlecorp/wick/commit/69d79c1c8eee66dcd766648c359145a1898691c7))
</details>

## v0.2.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-599514816356f7fab3b2122156092166f7815427/>
<csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-806afef0cbc45977d782e8a1b6d79ef6ca8c397d/>
<csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/>

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

## v0.2.0 (2023-04-19)

<csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/>
<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-7e2538202a03999c2b5781d7658b72118dce9446/>
<csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>

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

