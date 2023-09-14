# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.15.0 (2023-09-14)

### Chore

 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints

### Documentation

 - <csr-id-713fedac1fa8256f5b1421186acb6e11291cfc47/> gave generated operations in `wick new` component more helpful names

### New Features

 - <csr-id-398a034a3950c5b5dc95418248dfeb1f4f27f2bc/> added oci options so `wick reg pull` can pull more intuitively
 - <csr-id-2ce019fed2c7d9348c9c47d5221d322e700ce293/> added support for wasm imports
 - <csr-id-dc2b85758d0a4655eeb4351f153c72bfd59b5177/> added more progress events at startup
 - <csr-id-8760659095ce1f0f9a0bbd835bcf34827b21317c/> added __dirname, consolidated loose render events
 - <csr-id-429040070d92fac38b029ca670c5102c46ec3b62/> improved output of `wick list`

### Bug Fixes

 - <csr-id-1c93902b7ee9693eca9479cf07f9f5c3e8f620e9/> surfaced output/errors from completed triggers
 - <csr-id-7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6/> fixed config rendering within trigger operations

### Refactor

 - <csr-id-644c2ffde3be9b39bd087147d2e6599fbb6c1c85/> made generic Binding struct
 - <csr-id-43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5/> lowercased the start character of all log events
 - <csr-id-67740fc8d8543374ecbbe0198ba694bb543750c9/> formatting
 - <csr-id-a576880fa97834d9f89cfd7db4a42598b24fc02c/> moved wick bin files to root

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 16 calendar days.
 - 16 days passed between releases.
 - 14 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Added oci options so `wick reg pull` can pull more intuitively ([`398a034`](https://github.com/candlecorp/wick/commit/398a034a3950c5b5dc95418248dfeb1f4f27f2bc))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`4d6e3f4`](https://github.com/candlecorp/wick/commit/4d6e3f437964552cfd6917310c17548b12e83eaf))
    - Surfaced output/errors from completed triggers ([`1c93902`](https://github.com/candlecorp/wick/commit/1c93902b7ee9693eca9479cf07f9f5c3e8f620e9))
    - Added support for wasm imports ([`2ce019f`](https://github.com/candlecorp/wick/commit/2ce019fed2c7d9348c9c47d5221d322e700ce293))
    - Made generic Binding struct ([`644c2ff`](https://github.com/candlecorp/wick/commit/644c2ffde3be9b39bd087147d2e6599fbb6c1c85))
    - Migrated AsRef<str> to concrete types or Into<String> ([`60128f7`](https://github.com/candlecorp/wick/commit/60128f7707f2d2a537ffa32e24376f58d7faa7be))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Added more progress events at startup ([`dc2b857`](https://github.com/candlecorp/wick/commit/dc2b85758d0a4655eeb4351f153c72bfd59b5177))
    - Added __dirname, consolidated loose render events ([`8760659`](https://github.com/candlecorp/wick/commit/8760659095ce1f0f9a0bbd835bcf34827b21317c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Fixed config rendering within trigger operations ([`7d0a399`](https://github.com/candlecorp/wick/commit/7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6))
    - Gave generated operations in `wick new` component more helpful names ([`713feda`](https://github.com/candlecorp/wick/commit/713fedac1fa8256f5b1421186acb6e11291cfc47))
    - Lowercased the start character of all log events ([`43fa508`](https://github.com/candlecorp/wick/commit/43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5))
    - Formatting ([`67740fc`](https://github.com/candlecorp/wick/commit/67740fc8d8543374ecbbe0198ba694bb543750c9))
    - Improved output of `wick list` ([`4290400`](https://github.com/candlecorp/wick/commit/429040070d92fac38b029ca670c5102c46ec3b62))
    - Moved wick bin files to root ([`a576880`](https://github.com/candlecorp/wick/commit/a576880fa97834d9f89cfd7db4a42598b24fc02c))
</details>

## v0.14.0 (2023-08-28)

<csr-id-33b83d42f7a83e6ea81805f0ec0745654d12683f/>

### Refactor

 - <csr-id-33b83d42f7a83e6ea81805f0ec0745654d12683f/> moved wick bin files to root

## v0.13.0 (2023-08-23)

## v0.12.0 (2023-08-08)

## v0.11.0 (2023-07-31)

## v0.10.0 (2023-06-21)

## v0.9.0 (2023-06-12)

## v0.8.0 (2023-05-22)

## v0.7.0 (2023-05-05)

## v0.6.0 (2023-04-18)

## v0.5.0 (2023-03-24)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 723 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Moved vino-cli to subcrate ([`7013692`](https://github.com/candlecorp/wick/commit/70136922cb393806a89e1ebb16937ff36afec456))
    - Refactored MessagePayload->MessageTransport, OutputPayload-> versioned Output, vino-guest->vino-component ([`f0cc38b`](https://github.com/candlecorp/wick/commit/f0cc38b16bbe16c8ccbe8b4fd95437d2677f73fe))
    - First working pass at GrpcUrlProviders ([`5fedbfc`](https://github.com/candlecorp/wick/commit/5fedbfc29e5957a3b92d1b706865bb50b075fac1))
    - Migrated rpc to grpc and tonic, refactored providers ([`5873d90`](https://github.com/candlecorp/wick/commit/5873d900331b17c903389cfe8cba1607bcb83b94))
    - Code formatting ([`168adf2`](https://github.com/candlecorp/wick/commit/168adf2eab034fa1e1385eeb44103a227de223c5))
    - Refactored providers and ports ([`b7c809d`](https://github.com/candlecorp/wick/commit/b7c809de70c4b367412019563a4036d635d82e8e))
    - Defined manifest as widl, added codegen for manifest implementation, moved manifest to vino-manifest ([`f52b3fe`](https://github.com/candlecorp/wick/commit/f52b3fe3e189ae0e59a0edf997b1b9db49d3ff71))
    - Refactoring where APIs live across cli, host, and runtime ([`5ba98eb`](https://github.com/candlecorp/wick/commit/5ba98eb8bd8871a676b5c5165c567080ab0bacff))
    - Splitting out into crates ([`7dd8f29`](https://github.com/candlecorp/wick/commit/7dd8f299553fb2d5e50df180313255ed90b4a6f2))
    - Added vinox, refactoring to crates, added run/exec command, crud api yaml, RunConfig ([`4ed1f53`](https://github.com/candlecorp/wick/commit/4ed1f53825c336b8618a80af0276cfba48ad6f4d))
    - Fixing output ([`f1c6ccc`](https://github.com/candlecorp/wick/commit/f1c6cccfa02224c41a8d8f100f96997591ebd511))
    - Added outputpayload handling for exceptions ([`93501fd`](https://github.com/candlecorp/wick/commit/93501fd42b1e648dbd6246f0af86fbe71e1bda7a))
    - Added run command, tweaked logging ([`903e40d`](https://github.com/candlecorp/wick/commit/903e40d6e994c15d17c30293e46d0d24a2b443f3))
    - Converted core behavior to load subcommand ([`b65baec`](https://github.com/candlecorp/wick/commit/b65baece0e0ab625467c82e268372dcde6c944e0))
    - Updated logger ([`6fbfbbb`](https://github.com/candlecorp/wick/commit/6fbfbbb5a02c82b50bc990a6bd17a024f17c62e0))
    - Wasmcloud integration ([`aae2234`](https://github.com/candlecorp/wick/commit/aae22341504f51988894f634dbef96f74838f68b))
</details>

