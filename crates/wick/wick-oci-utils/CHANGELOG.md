# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.4.0 (2023-09-14)

### Chore

 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-f3904cfd28afb82fc727d096ef117c47e81b4160/> marked cache pull test as integration test

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-398a034a3950c5b5dc95418248dfeb1f4f27f2bc/> added oci options so `wick reg pull` can pull more intuitively
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-3ae02baaaa302521b0abc571ebdb08ae55a3a48e/> improved performance of frequently used regex
 - <csr-id-f4f04af492c7e0fe90472a6a5bafebfdbeddf622/> added types to package
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/> adding tar.gz for extra files

### Bug Fixes

 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-e107d7cc2fb3d36925fe8af471b164c07ec3e15d/> fixed broken cache path, fixed unrendered Volume configuraton
 - <csr-id-37f52c4bf5903d0e6be0e167846bc4aff64ed384/> fixed panic on invalid reference format, added candle reg as default domain
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-c61ca5f7320b533db3b69bdbe81fd37edbaa8eac/> multi-level recursive files in package
 - <csr-id-57698d4a6e4b86f5f438d12928ccdbbbb20a8abf/> fixed importing of assets from assets
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Refactor

 - <csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/> removed double parse and added cheaper function to check OCI references

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 24 commits contributed to the release over the course of 135 calendar days.
 - 148 days passed between releases.
 - 22 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#405](https://github.com/candlecorp/wick/issues/405)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#405](https://github.com/candlecorp/wick/issues/405)**
    - Fixed "refusing to overwrite ..." errors on application runs. ([`a10242d`](https://github.com/candlecorp/wick/commit/a10242d4786cfa199eaf61289b9da99d09c114a7))
 * **Uncategorized**
    - Added oci options so `wick reg pull` can pull more intuitively ([`398a034`](https://github.com/candlecorp/wick/commit/398a034a3950c5b5dc95418248dfeb1f4f27f2bc))
    - Migrated AsRef<str> to concrete types or Into<String> ([`60128f7`](https://github.com/candlecorp/wick/commit/60128f7707f2d2a537ffa32e24376f58d7faa7be))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Added `wick config expand` ([`33ea9cd`](https://github.com/candlecorp/wick/commit/33ea9cd5fff9a85398e7fc15661cb9401a085c18))
    - Removed double parse and added cheaper function to check OCI references ([`732db25`](https://github.com/candlecorp/wick/commit/732db25382951c1cb5c245af35dd3fcbf2677a71))
    - Improved performance of frequently used regex ([`3ae02ba`](https://github.com/candlecorp/wick/commit/3ae02baaaa302521b0abc571ebdb08ae55a3a48e))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Fixed included cached assets on wick reg push ([`4577461`](https://github.com/candlecorp/wick/commit/4577461e0a767ec99ae6482c2e2efeb3069ca0c8))
    - Marked cache pull test as integration test ([`f3904cf`](https://github.com/candlecorp/wick/commit/f3904cfd28afb82fc727d096ef117c47e81b4160))
    - Fixed broken cache path, fixed unrendered Volume configuraton ([`e107d7c`](https://github.com/candlecorp/wick/commit/e107d7cc2fb3d36925fe8af471b164c07ec3e15d))
    - Fixed panic on invalid reference format, added candle reg as default domain ([`37f52c4`](https://github.com/candlecorp/wick/commit/37f52c4bf5903d0e6be0e167846bc4aff64ed384))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Added types to package ([`f4f04af`](https://github.com/candlecorp/wick/commit/f4f04af492c7e0fe90472a6a5bafebfdbeddf622))
    - Multi-level recursive files in package ([`c61ca5f`](https://github.com/candlecorp/wick/commit/c61ca5f7320b533db3b69bdbe81fd37edbaa8eac))
    - Fixed importing of assets from assets ([`57698d4`](https://github.com/candlecorp/wick/commit/57698d4a6e4b86f5f438d12928ccdbbbb20a8abf))
    - Added glob support in package files ([`53ff1dd`](https://github.com/candlecorp/wick/commit/53ff1dd49057a0b7cb45deff02b350d8f1b2970e))
    - Fixed linting issues ([`6c6f9a8`](https://github.com/candlecorp/wick/commit/6c6f9a80f9873f5989453c7800a355724cb61fff))
    - Added gzip error handling ([`6fb111c`](https://github.com/candlecorp/wick/commit/6fb111cc0068ca5a4709ef274b046c0b590eee08))
    - Adding tar.gz for extra files ([`8c58c35`](https://github.com/candlecorp/wick/commit/8c58c354e765a51abb602b184c45055b9d561ed5))
</details>

## v0.3.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/>
<csr-id-f3904cfd28afb82fc727d096ef117c47e81b4160/>
<csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-f3904cfd28afb82fc727d096ef117c47e81b4160/> marked cache pull test as integration test

### New Features

 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-3ae02baaaa302521b0abc571ebdb08ae55a3a48e/> improved performance of frequently used regex
 - <csr-id-f4f04af492c7e0fe90472a6a5bafebfdbeddf622/> added types to package
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/> adding tar.gz for extra files

### Bug Fixes

 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-e107d7cc2fb3d36925fe8af471b164c07ec3e15d/> fixed broken cache path, fixed unrendered Volume configuraton
 - <csr-id-37f52c4bf5903d0e6be0e167846bc4aff64ed384/> fixed panic on invalid reference format, added candle reg as default domain
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-c61ca5f7320b533db3b69bdbe81fd37edbaa8eac/> multi-level recursive files in package
 - <csr-id-57698d4a6e4b86f5f438d12928ccdbbbb20a8abf/> fixed importing of assets from assets
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Refactor

 - <csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/> removed double parse and added cheaper function to check OCI references

## v0.3.0 (2023-04-18)

<csr-id-7361b149ca108904341364426e1509105913f31f/>
<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/>
<csr-id-7e2538202a03999c2b5781d7658b72118dce9446/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>
<csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/>

### Chore

 - <csr-id-7361b149ca108904341364426e1509105913f31f/> release
   flow-component, flow-expression-parser, flow-graph, wick-asset-reference, wick-component, wick-config, wick-oci-utils
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al

### Refactor

 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup
 - <csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/> added registry tests, invoke tests, v1 tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 38 calendar days.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Release ([`7361b14`](https://github.com/candlecorp/wick/commit/7361b149ca108904341364426e1509105913f31f))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Added registry tests, invoke tests, v1 tests ([`3802bf9`](https://github.com/candlecorp/wick/commit/3802bf93746725527d5dfa80f3c65d3314d4122c))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

