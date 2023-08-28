# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.16.0 (2023-08-28)

<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-599514816356f7fab3b2122156092166f7815427/>
<csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/>
<csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/>
<csr-id-f18a77df361bfceac2ca0ed00c0d39db45b624ee/>

### Chore

 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings

### Chore

 - <csr-id-e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa/> generated changelogs

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-0489b158b2441fb5971383a77e2f3fb5589bdd56/> updated template to avoid footguns
 - <csr-id-32fcf67e9ba9c50695a5ee11e50b6674c5fdde96/> added wasi test component, added wick-operation proc_macros and adapter macros
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-454287903fff88c9219860f35d82ba753d659a84/> added raw call support to wasm codegen, added formatter for codegen
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-a4160219ac2ba43cee39d31721eaf2821cd7906b/> made Base64Bytes the primary bytes struct, updated liquid_json
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-703988e288b32a1dc7f3d9dee232f4b4c79cc1cc/> made CLI parsing of arguments slightly smarter
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-695dfdb64e77bfb61152594749b3c6afad8d05cf/> qol improvements to wick_component, made wasmrs invisible
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-33c82afccdbcb4d7cda43e0ae880381501668478/> propagated seed to component context
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component

### Bug Fixes

 - <csr-id-d64a3550825b67e0cc59e3c44e3eca42b24c9cab/> converted all level spans to info_spans
 - <csr-id-3208691ffb824e9f83d9845ae274c9b60bb8d4fa/> converted all level spans to info_spans
 - <csr-id-f12a7902c0a26760155c74119adf229799aaf835/> fixed unnecessary lifetime that could cause an error on build
 - <csr-id-d901966927c3eec44270bbd2cd5d84baaa1f3462/> fixed relative volumes again
 - <csr-id-242efac7ca4166ee0ad05600c4d697ab1db90b2e/> fixed bail action on propagate_if_error macro
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-d3b4b02214d01cdc338cfb88a22f904bbb719134/> handled unwraps that led to in-wasm panics

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-f18a77df361bfceac2ca0ed00c0d39db45b624ee/> fixed unnecessary lifetime warning

### New Features (BREAKING)

 - <csr-id-34e1484443de014ebe010063640f937e528df10a/> changed pre-request middleware to one output union vs a request/response race

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 34 commits contributed to the release over the course of 123 calendar days.
 - 131 days passed between releases.
 - 33 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#278](https://github.com/candlecorp/wick/issues/278), [#341](https://github.com/candlecorp/wick/issues/341), [#375](https://github.com/candlecorp/wick/issues/375), [#388](https://github.com/candlecorp/wick/issues/388)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#278](https://github.com/candlecorp/wick/issues/278)**
    - Handled unwraps that led to in-wasm panics ([`d3b4b02`](https://github.com/candlecorp/wick/commit/d3b4b02214d01cdc338cfb88a22f904bbb719134))
 * **[#341](https://github.com/candlecorp/wick/issues/341)**
    - Added ctx.inherent.timestamp, improved error message output ([`efe6055`](https://github.com/candlecorp/wick/commit/efe605510b846d2556f6060ba710fa154bdca7c4))
 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **[#388](https://github.com/candlecorp/wick/issues/388)**
    - Added `wick install` ([`3158048`](https://github.com/candlecorp/wick/commit/3158048ad1d0c33518cb647d08f927606afcecd0))
 * **Uncategorized**
    - Generated changelogs ([`e1d6c05`](https://github.com/candlecorp/wick/commit/e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa))
    - Updated template to avoid footguns ([`0489b15`](https://github.com/candlecorp/wick/commit/0489b158b2441fb5971383a77e2f3fb5589bdd56))
    - Converted all level spans to info_spans ([`d64a355`](https://github.com/candlecorp/wick/commit/d64a3550825b67e0cc59e3c44e3eca42b24c9cab))
    - Converted all level spans to info_spans ([`3208691`](https://github.com/candlecorp/wick/commit/3208691ffb824e9f83d9845ae274c9b60bb8d4fa))
    - Fixed unnecessary lifetime that could cause an error on build ([`f12a790`](https://github.com/candlecorp/wick/commit/f12a7902c0a26760155c74119adf229799aaf835))
    - Fixed relative volumes again ([`d901966`](https://github.com/candlecorp/wick/commit/d901966927c3eec44270bbd2cd5d84baaa1f3462))
    - Fixed unnecessary lifetime warning ([`f18a77d`](https://github.com/candlecorp/wick/commit/f18a77df361bfceac2ca0ed00c0d39db45b624ee))
    - Added wasi test component, added wick-operation proc_macros and adapter macros ([`32fcf67`](https://github.com/candlecorp/wick/commit/32fcf67e9ba9c50695a5ee11e50b6674c5fdde96))
    - Re-added exposing volumes to WASI components ([`ce9d202`](https://github.com/candlecorp/wick/commit/ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85))
    - Added raw call support to wasm codegen, added formatter for codegen ([`4542879`](https://github.com/candlecorp/wick/commit/454287903fff88c9219860f35d82ba753d659a84))
    - Changed pre-request middleware to one output union vs a request/response race ([`34e1484`](https://github.com/candlecorp/wick/commit/34e1484443de014ebe010063640f937e528df10a))
    - Fixed bail action on propagate_if_error macro ([`242efac`](https://github.com/candlecorp/wick/commit/242efac7ca4166ee0ad05600c4d697ab1db90b2e))
    - Made Base64Bytes the primary bytes struct, updated liquid_json ([`a416021`](https://github.com/candlecorp/wick/commit/a4160219ac2ba43cee39d31721eaf2821cd7906b))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Added on_error & transaction support to ms sql server SQL implementation ([`d85d6f5`](https://github.com/candlecorp/wick/commit/d85d6f568d4548036c1af61e515c3fc187be6a6e))
    - Made CLI parsing of arguments slightly smarter ([`703988e`](https://github.com/candlecorp/wick/commit/703988e288b32a1dc7f3d9dee232f4b4c79cc1cc))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Qol improvements to wick_component, made wasmrs invisible ([`695dfdb`](https://github.com/candlecorp/wick/commit/695dfdb64e77bfb61152594749b3c6afad8d05cf))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Updated to rust 1.69.0, fixed associated warnings ([`e561fd6`](https://github.com/candlecorp/wick/commit/e561fd668afb1e1af3639c472a893b7fcfe2bf54))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Propagated seed to component context ([`33c82af`](https://github.com/candlecorp/wick/commit/33c82afccdbcb4d7cda43e0ae880381501668478))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
</details>

## v0.15.0 (2023-04-18)

<csr-id-7361b149ca108904341364426e1509105913f31f/>
<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>

### Chore

 - <csr-id-7361b149ca108904341364426e1509105913f31f/> release
   flow-component, flow-expression-parser, flow-graph, wick-asset-reference, wick-component, wick-config, wick-oci-utils
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 1 calendar day.
 - 26 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`7361b14`](https://github.com/candlecorp/wick/commit/7361b149ca108904341364426e1509105913f31f))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
</details>

## v0.14.0 (2023-03-23)

<csr-id-501d6056a5ff2d06290f88f73885c6c12afd77e9/>

### Chore

 - <csr-id-501d6056a5ff2d06290f88f73885c6c12afd77e9/> Release

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`501d605`](https://github.com/candlecorp/wick/commit/501d6056a5ff2d06290f88f73885c6c12afd77e9))
</details>

## v0.13.0 (2023-03-23)

<csr-id-f229d8ee9dbb1c051d18b911bb4ef868b968ea14/>
<csr-id-406c10999648ca923fc8994b5835d11c823c19ce/>

### Chore

 - <csr-id-f229d8ee9dbb1c051d18b911bb4ef868b968ea14/> Release
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages

### New Features

 - <csr-id-39fb923c30ec819bcbe665ef4fad569eebdfe194/> substreams/bracketing + codegen improvements
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 2 calendar days.
 - 8 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`f229d8e`](https://github.com/candlecorp/wick/commit/f229d8ee9dbb1c051d18b911bb4ef868b968ea14))
    - Substreams/bracketing + codegen improvements ([`39fb923`](https://github.com/candlecorp/wick/commit/39fb923c30ec819bcbe665ef4fad569eebdfe194))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
</details>

## v0.12.0 (2023-03-15)

### New Features

 - <csr-id-8745221bb0e25332f85bebe2387bc10a440ed5ac/> added codegen based off component.yaml

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Added codegen based off component.yaml ([`8745221`](https://github.com/candlecorp/wick/commit/8745221bb0e25332f85bebe2387bc10a440ed5ac))
</details>

