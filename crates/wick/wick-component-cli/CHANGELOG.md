# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### Chore

 - <csr-id-e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa/> generated changelogs

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-703988e288b32a1dc7f3d9dee232f4b4c79cc1cc/> made CLI parsing of arguments slightly smarter
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/> added settings file, wick reg login, & wick reg push --latest
 - <csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/> add Base64Bytes to wick-packet

### Bug Fixes

 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/> removed conflicting timeouts in favor of per-op timeouts

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 123 calendar days.
 - 131 days passed between releases.
 - 17 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#399](https://github.com/candlecorp/wick/issues/399)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#399](https://github.com/candlecorp/wick/issues/399)**
    - Better http client substream support. ([`744f1ac`](https://github.com/candlecorp/wick/commit/744f1ac3d5fa8c28e8e0a1e80d7f5e49839c0c43))
 * **Uncategorized**
    - Generated changelogs ([`e1d6c05`](https://github.com/candlecorp/wick/commit/e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa))
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Eliminated fetching of bytes before checking cache ([`586ace0`](https://github.com/candlecorp/wick/commit/586ace0978ca8adf58bf4d1fa5ed392015297c21))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Removed conflicting timeouts in favor of per-op timeouts ([`888814b`](https://github.com/candlecorp/wick/commit/888814bb24d3d4dd4b460af2616a72814f2bd7a1))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Made CLI parsing of arguments slightly smarter ([`703988e`](https://github.com/candlecorp/wick/commit/703988e288b32a1dc7f3d9dee232f4b4c79cc1cc))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Added settings file, wick reg login, & wick reg push --latest ([`63858e1`](https://github.com/candlecorp/wick/commit/63858e1bc6673b61d50fa8f66dc4378369850910))
    - Add Base64Bytes to wick-packet ([`399c5d5`](https://github.com/candlecorp/wick/commit/399c5d518b0a291dba63fb3f69337af2911d1776))
</details>

## v0.1.0 (2023-04-19)

<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-7e2538202a03999c2b5781d7658b72118dce9446/>
<csr-id-890b9dd879e9d18c8e989989a01e73eb5a987b2f/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>

### Chore

 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/> renamed existing wafl references

### New Features

 - <csr-id-b83485305d609f9f599ae4a3f0aa03d9e101fb5c/> centralized relative file resolution within wick-config
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger
 - <csr-id-8745221bb0e25332f85bebe2387bc10a440ed5ac/> added codegen based off component.yaml

### Bug Fixes

 - <csr-id-1c58123f86ec95073b503790fe272b04003a05df/> adjusted default features on deps
 - <csr-id-16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc/> path resolution and missing wasm components in interpreter

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable
 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils

### Test

 - <csr-id-890b9dd879e9d18c8e989989a01e73eb5a987b2f/> moved tests with native-component to separate rust project in test/integration
 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 17 commits contributed to the release over the course of 39 calendar days.
 - 14 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Moved tests with native-component to separate rust project in test/integration ([`890b9dd`](https://github.com/candlecorp/wick/commit/890b9dd879e9d18c8e989989a01e73eb5a987b2f))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Adjusted default features on deps ([`1c58123`](https://github.com/candlecorp/wick/commit/1c58123f86ec95073b503790fe272b04003a05df))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
    - Path resolution and missing wasm components in interpreter ([`16bb6b4`](https://github.com/candlecorp/wick/commit/16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc))
    - Centralized relative file resolution within wick-config ([`b834853`](https://github.com/candlecorp/wick/commit/b83485305d609f9f599ae4a3f0aa03d9e101fb5c))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed existing wafl references ([`3a42e63`](https://github.com/candlecorp/wick/commit/3a42e6388e3561103412ca3e47db8b5feb5ef3a9))
    - Added codegen based off component.yaml ([`8745221`](https://github.com/candlecorp/wick/commit/8745221bb0e25332f85bebe2387bc10a440ed5ac))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

