# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.23.0 (2023-10-18)

### Chore

 - <csr-id-35ff51b8a93c27475765a7eb65c23256f4f93d67/> updated versions and changelogs
 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-3580951b5faa8ef279291e5a6f994d1c9e0785d6/> cleaned up legacy naming

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-04d4fb0fc7137946fa10ee3e0f0be4c0cc73c8b3/> added ability to pass `with:` config to switch case operations
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 19 commits contributed to the release over the course of 180 calendar days.
 - 182 days passed between releases.
 - 18 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#232](https://github.com/candlecorp/wick/issues/232), [#319](https://github.com/candlecorp/wick/issues/319)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **Uncategorized**
    - Updated versions and changelogs ([`35ff51b`](https://github.com/candlecorp/wick/commit/35ff51b8a93c27475765a7eb65c23256f4f93d67))
    - Migrated AsRef<str> to concrete types or Into<String> ([`60128f7`](https://github.com/candlecorp/wick/commit/60128f7707f2d2a537ffa32e24376f58d7faa7be))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Added flow sequences, enhanced port inference ([`2a5cf0c`](https://github.com/candlecorp/wick/commit/2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695))
    - Cleaned up legacy naming ([`3580951`](https://github.com/candlecorp/wick/commit/3580951b5faa8ef279291e5a6f994d1c9e0785d6))
    - Added ability to pass `with:` config to switch case operations ([`04d4fb0`](https://github.com/candlecorp/wick/commit/04d4fb0fc7137946fa10ee3e0f0be4c0cc73c8b3))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Added pluck shorthand where e.g. `op.name.input -> op.name` ([`262e0b5`](https://github.com/candlecorp/wick/commit/262e0b50c84229872ce7d1f006a878281b46d8e9))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Added the ability to create inline node IDs in flow config ([`f7d7274`](https://github.com/candlecorp/wick/commit/f7d72741adae67477634ccdf52b93fe8f0c3c35f))
</details>

## v0.22.0 (2023-09-14)

<csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/>
<csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-3580951b5faa8ef279291e5a6f994d1c9e0785d6/>

### Chore

 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-3580951b5faa8ef279291e5a6f994d1c9e0785d6/> cleaned up legacy naming

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-04d4fb0fc7137946fa10ee3e0f0be4c0cc73c8b3/> added ability to pass `with:` config to switch case operations
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

## v0.21.1 (2023-08-28)

<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-3580951b5faa8ef279291e5a6f994d1c9e0785d6/>

### Chore

 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-3580951b5faa8ef279291e5a6f994d1c9e0785d6/> cleaned up legacy naming

### New Features

 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-04d4fb0fc7137946fa10ee3e0f0be4c0cc73c8b3/> added ability to pass `with:` config to switch case operations
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

## v0.21.0 (2023-04-18)

<csr-id-7361b149ca108904341364426e1509105913f31f/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-406c10999648ca923fc8994b5835d11c823c19ce/>
<csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/>
<csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/>
<csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/>

### Chore

 - <csr-id-7361b149ca108904341364426e1509105913f31f/> release
   flow-component, flow-expression-parser, flow-graph, wick-asset-reference, wick-component, wick-config, wick-oci-utils
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages
 - <csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/> renamed existing wafl references

### New Features

 - <csr-id-b83485305d609f9f599ae4a3f0aa03d9e101fb5c/> centralized relative file resolution within wick-config
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Refactor

 - <csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/> centralized APIs around configuration yaml

### Refactor (BREAKING)

 - <csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/> removed "default" value substitution in favor of a future impl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release over the course of 38 calendar days.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`7361b14`](https://github.com/candlecorp/wick/commit/7361b149ca108904341364426e1509105913f31f))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Centralized relative file resolution within wick-config ([`b834853`](https://github.com/candlecorp/wick/commit/b83485305d609f9f599ae4a3f0aa03d9e101fb5c))
    - Centralized APIs around configuration yaml ([`fd71df4`](https://github.com/candlecorp/wick/commit/fd71df4baaa3f856454624396eff9d9ee8c4473f))
    - Removed "default" value substitution in favor of a future impl ([`c7b84da`](https://github.com/candlecorp/wick/commit/c7b84daacad21d9ba2c44123a6b0695db3b43528))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed existing wafl references ([`3a42e63`](https://github.com/candlecorp/wick/commit/3a42e6388e3561103412ca3e47db8b5feb5ef3a9))
    - Incorporated interface.json into component.yaml ([`97280ee`](https://github.com/candlecorp/wick/commit/97280ee71b361472dbb6ae32c77626b07c218554))
    - Renamed wick-config-component to wick-config, added app config, restructured triggers, added trigger test component ([`24ef43f`](https://github.com/candlecorp/wick/commit/24ef43f7fc978c1f33f27a1e90f9971abdeb9b11))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

