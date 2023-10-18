# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0 (2023-10-18)

### New Features

 - <csr-id-29a4831b7629d1e68bb07a54ec278a3ebab0f79d/> added event-stream handling to http client and raw router
 - <csr-id-11449d002b80fbc22ec5e4b684b09fbcc949a9c7/> added support for wasm component-model triggers

### Bug Fixes

 - <csr-id-42932f4f0febf3d5398f2ff8edb7d8d9761a9842/> ensure stream is done

### Refactor

 - <csr-id-051de9cd392b4625ff4964ff08582767ca1dc3fe/> removed spawns in favor of stream chains
 - <csr-id-378c726823ec2fe65a168d7e205ea613b2b1c1b3/> unified input/output structs for all calls
 - <csr-id-69d79c1c8eee66dcd766648c359145a1898691c7/> removed native stdlib and associated references
 - <csr-id-42a39c2b9150b56e27c8b7b41cccebc0cef09015/> pulled triggers into their own crates

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 5 calendar days.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Removed spawns in favor of stream chains ([`051de9c`](https://github.com/candlecorp/wick/commit/051de9cd392b4625ff4964ff08582767ca1dc3fe))
    - Ensure stream is done ([`42932f4`](https://github.com/candlecorp/wick/commit/42932f4f0febf3d5398f2ff8edb7d8d9761a9842))
    - Added event-stream handling to http client and raw router ([`29a4831`](https://github.com/candlecorp/wick/commit/29a4831b7629d1e68bb07a54ec278a3ebab0f79d))
    - Unified input/output structs for all calls ([`378c726`](https://github.com/candlecorp/wick/commit/378c726823ec2fe65a168d7e205ea613b2b1c1b3))
    - Removed native stdlib and associated references ([`69d79c1`](https://github.com/candlecorp/wick/commit/69d79c1c8eee66dcd766648c359145a1898691c7))
    - Added support for wasm component-model triggers ([`11449d0`](https://github.com/candlecorp/wick/commit/11449d002b80fbc22ec5e4b684b09fbcc949a9c7))
    - Pulled triggers into their own crates ([`42a39c2`](https://github.com/candlecorp/wick/commit/42a39c2b9150b56e27c8b7b41cccebc0cef09015))
</details>

