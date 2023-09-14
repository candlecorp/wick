# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.2.0 (2023-09-14)

### Chore

 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog
 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports

### Bug Fixes

 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

### Refactor

 - <csr-id-0f3fef30abf88525a9966b823edccb18a1919aaf/> removed mutexes in PacketStream, made Invocation state error-proof
 - <csr-id-43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5/> lowercased the start character of all log events
 - <csr-id-4b9d70f04dbc80eff5df9fa50b6a0d065f652af1/> extracted generated proto code to a separate task and remove build script
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 24 commits contributed to the release over the course of 143 calendar days.
 - 148 days passed between releases.
 - 23 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#319](https://github.com/candlecorp/wick/issues/319)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **Uncategorized**
    - Removed mutexes in PacketStream, made Invocation state error-proof ([`0f3fef3`](https://github.com/candlecorp/wick/commit/0f3fef30abf88525a9966b823edccb18a1919aaf))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Lowercased the start character of all log events ([`43fa508`](https://github.com/candlecorp/wick/commit/43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Extracted generated proto code to a separate task and remove build script ([`4b9d70f`](https://github.com/candlecorp/wick/commit/4b9d70f04dbc80eff5df9fa50b6a0d065f652af1))
    - Added unions to type definitions ([`222cc7f`](https://github.com/candlecorp/wick/commit/222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Updated to rust 1.69.0, fixed associated warnings ([`e561fd6`](https://github.com/candlecorp/wick/commit/e561fd668afb1e1af3639c472a893b7fcfe2bf54))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Added restapi router ([`58045d0`](https://github.com/candlecorp/wick/commit/58045d0fe75f519b84ebd45f3b1493e55fd4b282))
    - Added proper type defs into config, closes #200. Fixed #228, #227 ([`49a53de`](https://github.com/candlecorp/wick/commit/49a53de6cb6631e2dc1f1e633d1c29d0510383cb))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added type imports ([`17c9058`](https://github.com/candlecorp/wick/commit/17c9058b98935fa8ed29dbc27b899c9e3244eb67))
</details>

## v0.1.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-599514816356f7fab3b2122156092166f7815427/>
<csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/>
<csr-id-4b9d70f04dbc80eff5df9fa50b6a0d065f652af1/>
<csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports

### Bug Fixes

 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

### Refactor

 - <csr-id-4b9d70f04dbc80eff5df9fa50b6a0d065f652af1/> extracted generated proto code to a separate task and remove build script
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation

## v0.1.0 (2023-04-18)

<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-c724b06b8cf7776ba48b5a799d9e04e074d1c99d/>
<csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/>
<csr-id-406c10999648ca923fc8994b5835d11c823c19ce/>
<csr-id-c27131154861be5625b82e1e7d99d8228df1fa39/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>

### Chore

 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-c724b06b8cf7776ba48b5a799d9e04e074d1c99d/> bumped deps, deprecated old crates, removed old kv component
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Bug Fixes

 - <csr-id-05d2aad6728b30a866d6fd30dfadc6626449ea38/> updated build job to defer to rust-toolchain.toml

### Other

 - <csr-id-c27131154861be5625b82e1e7d99d8228df1fa39/> added http types example as tests

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 38 calendar days.
 - 13 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Updated build job to defer to rust-toolchain.toml ([`05d2aad`](https://github.com/candlecorp/wick/commit/05d2aad6728b30a866d6fd30dfadc6626449ea38))
    - Bumped deps, deprecated old crates, removed old kv component ([`c724b06`](https://github.com/candlecorp/wick/commit/c724b06b8cf7776ba48b5a799d9e04e074d1c99d))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Added http types example as tests ([`c271311`](https://github.com/candlecorp/wick/commit/c27131154861be5625b82e1e7d99d8228df1fa39))
    - Incorporated interface.json into component.yaml ([`97280ee`](https://github.com/candlecorp/wick/commit/97280ee71b361472dbb6ae32c77626b07c218554))
    - Shoring up tests. fixed error propagation and hung txs stemming from timeouts ([`46310b9`](https://github.com/candlecorp/wick/commit/46310b98b6933c5a6d84c32863391bb482af5ac3))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

