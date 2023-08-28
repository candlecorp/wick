# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.6.0 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/>
<csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/>
<csr-id-39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/> fixed warnings

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog
 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-e5ed32378e0fd61c8bb1560027d252c0c93059a1/> added wick config dotviz, made interpreter tolerant of unused ports
 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-bd8af683437d46ed7281fd8cd806efe22ffa0f6f/> added quote-delimeted paths to field syntax, made rest router return errors on error packets
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-51d1da4a4ac6908fd1041ffd14ac7387b80b8ff6/> added arbitrary length plucked paths w/ support for array indices
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-5f59bb11179ee19f49c82159e3b34f3abfe1c5ab/> fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f/> cleaned up intepreter, made some errors/warnings more clear

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 20 commits contributed to the release over the course of 130 calendar days.
 - 131 days passed between releases.
 - 20 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#232](https://github.com/candlecorp/wick/issues/232), [#319](https://github.com/candlecorp/wick/issues/319), [#388](https://github.com/candlecorp/wick/issues/388)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#388](https://github.com/candlecorp/wick/issues/388)**
    - Added `wick install` ([`3158048`](https://github.com/candlecorp/wick/commit/3158048ad1d0c33518cb647d08f927606afcecd0))
 * **Uncategorized**
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Added `wick config expand` ([`33ea9cd`](https://github.com/candlecorp/wick/commit/33ea9cd5fff9a85398e7fc15661cb9401a085c18))
    - Added wick config dotviz, made interpreter tolerant of unused ports ([`e5ed323`](https://github.com/candlecorp/wick/commit/e5ed32378e0fd61c8bb1560027d252c0c93059a1))
    - Added flow sequences, enhanced port inference ([`2a5cf0c`](https://github.com/candlecorp/wick/commit/2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695))
    - Cleaned up intepreter, made some errors/warnings more clear ([`39f6a7d`](https://github.com/candlecorp/wick/commit/39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb ([`5f59bb1`](https://github.com/candlecorp/wick/commit/5f59bb11179ee19f49c82159e3b34f3abfe1c5ab))
    - Added quote-delimeted paths to field syntax, made rest router return errors on error packets ([`bd8af68`](https://github.com/candlecorp/wick/commit/bd8af683437d46ed7281fd8cd806efe22ffa0f6f))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Added arbitrary length plucked paths w/ support for array indices ([`51d1da4`](https://github.com/candlecorp/wick/commit/51d1da4a4ac6908fd1041ffd14ac7387b80b8ff6))
    - Added pluck shorthand where e.g. `op.name.input -> op.name` ([`262e0b5`](https://github.com/candlecorp/wick/commit/262e0b50c84229872ce7d1f006a878281b46d8e9))
    - Fixed warnings ([`ab7d535`](https://github.com/candlecorp/wick/commit/ab7d5355945adb592c4e00ccdc8b268e146e6535))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Added the ability to create inline node IDs in flow config ([`f7d7274`](https://github.com/candlecorp/wick/commit/f7d72741adae67477634ccdf52b93fe8f0c3c35f))
</details>

## v0.5.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/>
<csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/>
<csr-id-39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/> fixed warnings

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-e5ed32378e0fd61c8bb1560027d252c0c93059a1/> added wick config dotviz, made interpreter tolerant of unused ports
 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-bd8af683437d46ed7281fd8cd806efe22ffa0f6f/> added quote-delimeted paths to field syntax, made rest router return errors on error packets
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-51d1da4a4ac6908fd1041ffd14ac7387b80b8ff6/> added arbitrary length plucked paths w/ support for array indices
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-5f59bb11179ee19f49c82159e3b34f3abfe1c5ab/> fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f/> cleaned up intepreter, made some errors/warnings more clear

## v0.5.0 (2023-04-18)

<csr-id-7361b149ca108904341364426e1509105913f31f/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>

### Chore

 - <csr-id-7361b149ca108904341364426e1509105913f31f/> release
   flow-component, flow-expression-parser, flow-graph, wick-asset-reference, wick-component, wick-config, wick-oci-utils
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 26 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`7361b14`](https://github.com/candlecorp/wick/commit/7361b149ca108904341364426e1509105913f31f))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
</details>

## v0.4.0 (2023-03-23)

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

## v0.3.0 (2023-03-23)

<csr-id-f229d8ee9dbb1c051d18b911bb4ef868b968ea14/>
<csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/>

### Chore

 - <csr-id-f229d8ee9dbb1c051d18b911bb4ef868b968ea14/> Release

### Refactor (BREAKING)

 - <csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/> removed "default" value substitution in favor of a future impl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 8 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`f229d8e`](https://github.com/candlecorp/wick/commit/f229d8ee9dbb1c051d18b911bb4ef868b968ea14))
    - Removed "default" value substitution in favor of a future impl ([`c7b84da`](https://github.com/candlecorp/wick/commit/c7b84daacad21d9ba2c44123a6b0695db3b43528))
</details>

## v0.2.0 (2023-03-15)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 4 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

