# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.2.0 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-4f3c0a02502098e8252613dbe3f8ee002da8382b/>
<csr-id-a0d92a6462f139a598be39decd633ceb7a956113/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-4f3c0a02502098e8252613dbe3f8ee002da8382b/> updated dependencies
 - <csr-id-a0d92a6462f139a598be39decd633ceb7a956113/> disabled some default features on wasmtime

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-b679aad2e505e2e4b15794dc4decc98c51aee077/> added v1 wasm signatures, bumped wasmrs, enabled module cache
 - <csr-id-3eb6ac3742b7cebaff7cf5dbf3e552cc6cd784f3/> reused wasmtime engine from runtime, updated wasm parser

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 47 calendar days.
 - 131 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#375](https://github.com/candlecorp/wick/issues/375)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **Uncategorized**
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Updated dependencies ([`4f3c0a0`](https://github.com/candlecorp/wick/commit/4f3c0a02502098e8252613dbe3f8ee002da8382b))
    - Disabled some default features on wasmtime ([`a0d92a6`](https://github.com/candlecorp/wick/commit/a0d92a6462f139a598be39decd633ceb7a956113))
    - Added v1 wasm signatures, bumped wasmrs, enabled module cache ([`b679aad`](https://github.com/candlecorp/wick/commit/b679aad2e505e2e4b15794dc4decc98c51aee077))
    - Reused wasmtime engine from runtime, updated wasm parser ([`3eb6ac3`](https://github.com/candlecorp/wick/commit/3eb6ac3742b7cebaff7cf5dbf3e552cc6cd784f3))
</details>

## v0.1.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-4f3c0a02502098e8252613dbe3f8ee002da8382b/>
<csr-id-a0d92a6462f139a598be39decd633ceb7a956113/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-4f3c0a02502098e8252613dbe3f8ee002da8382b/> updated dependencies
 - <csr-id-a0d92a6462f139a598be39decd633ceb7a956113/> disabled some default features on wasmtime

### New Features

 - <csr-id-b679aad2e505e2e4b15794dc4decc98c51aee077/> added v1 wasm signatures, bumped wasmrs, enabled module cache
 - <csr-id-3eb6ac3742b7cebaff7cf5dbf3e552cc6cd784f3/> reused wasmtime engine from runtime, updated wasm parser

## v0.1.0 (2023-04-19)

<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>

### Chore

 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 39 calendar days.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

