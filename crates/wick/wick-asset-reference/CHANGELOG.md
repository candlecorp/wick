# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.5.0 (2023-10-18)

### Chore

 - <csr-id-35ff51b8a93c27475765a7eb65c23256f4f93d67/> updated versions and changelogs
 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-85ba75c8696fb8197c70b891f31724a3b069d59c/> formatting
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-7b60a70188be0c9be39138accee9329a810fc1e5/> implemented config cache
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/> added asset flags, fixed relative volumes, fixed manifest locations
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports

### Bug Fixes

 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-46d29109dc6502ea826236cf5438c54e02674d04/> fixed package push trying to retrieve remote assets
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-28c625552c460ac5c337efad3b0d621c9ec593cc/> don't fetch lazy assets
 - <csr-id-c1bb1d409adbf77c59da9e3241fa23d90cc39c8e/> added config fetch_all to fetch everything & made the default lazy
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-f791c68116bc2c9d7a57c4f5d61fbaddfeb4bd41/> pulled triggers out of runtime
 - <csr-id-a576880fa97834d9f89cfd7db4a42598b24fc02c/> moved wick bin files to root
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/> removed double parse and added cheaper function to check OCI references

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 32 commits contributed to the release over the course of 176 calendar days.
 - 182 days passed between releases.
 - 30 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#319](https://github.com/candlecorp/wick/issues/319), [#388](https://github.com/candlecorp/wick/issues/388), [#405](https://github.com/candlecorp/wick/issues/405)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#388](https://github.com/candlecorp/wick/issues/388)**
    - Added `wick install` ([`3158048`](https://github.com/candlecorp/wick/commit/3158048ad1d0c33518cb647d08f927606afcecd0))
 * **[#405](https://github.com/candlecorp/wick/issues/405)**
    - Fixed "refusing to overwrite ..." errors on application runs. ([`a10242d`](https://github.com/candlecorp/wick/commit/a10242d4786cfa199eaf61289b9da99d09c114a7))
 * **Uncategorized**
    - Pulled triggers out of runtime ([`f791c68`](https://github.com/candlecorp/wick/commit/f791c68116bc2c9d7a57c4f5d61fbaddfeb4bd41))
    - Updated versions and changelogs ([`35ff51b`](https://github.com/candlecorp/wick/commit/35ff51b8a93c27475765a7eb65c23256f4f93d67))
    - Migrated AsRef<str> to concrete types or Into<String> ([`60128f7`](https://github.com/candlecorp/wick/commit/60128f7707f2d2a537ffa32e24376f58d7faa7be))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Formatting ([`85ba75c`](https://github.com/candlecorp/wick/commit/85ba75c8696fb8197c70b891f31724a3b069d59c))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Moved wick bin files to root ([`a576880`](https://github.com/candlecorp/wick/commit/a576880fa97834d9f89cfd7db4a42598b24fc02c))
    - Implemented config cache ([`7b60a70`](https://github.com/candlecorp/wick/commit/7b60a70188be0c9be39138accee9329a810fc1e5))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax ([`b0b9cd2`](https://github.com/candlecorp/wick/commit/b0b9cd20f748ffe1956ad2501fe23991fededf13))
    - Added `wick config expand` ([`33ea9cd`](https://github.com/candlecorp/wick/commit/33ea9cd5fff9a85398e7fc15661cb9401a085c18))
    - Eliminated fetching of bytes before checking cache ([`586ace0`](https://github.com/candlecorp/wick/commit/586ace0978ca8adf58bf4d1fa5ed392015297c21))
    - Removed double parse and added cheaper function to check OCI references ([`732db25`](https://github.com/candlecorp/wick/commit/732db25382951c1cb5c245af35dd3fcbf2677a71))
    - Fixed included cached assets on wick reg push ([`4577461`](https://github.com/candlecorp/wick/commit/4577461e0a767ec99ae6482c2e2efeb3069ca0c8))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Fixed package push trying to retrieve remote assets ([`46d2910`](https://github.com/candlecorp/wick/commit/46d29109dc6502ea826236cf5438c54e02674d04))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Added restapi router ([`58045d0`](https://github.com/candlecorp/wick/commit/58045d0fe75f519b84ebd45f3b1493e55fd4b282))
    - Don't fetch lazy assets ([`28c6255`](https://github.com/candlecorp/wick/commit/28c625552c460ac5c337efad3b0d621c9ec593cc))
    - Added asset flags, fixed relative volumes, fixed manifest locations ([`3dd4cdb`](https://github.com/candlecorp/wick/commit/3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08))
    - Added config fetch_all to fetch everything & made the default lazy ([`c1bb1d4`](https://github.com/candlecorp/wick/commit/c1bb1d409adbf77c59da9e3241fa23d90cc39c8e))
    - Added glob support in package files ([`53ff1dd`](https://github.com/candlecorp/wick/commit/53ff1dd49057a0b7cb45deff02b350d8f1b2970e))
    - Fixed linting issues ([`6c6f9a8`](https://github.com/candlecorp/wick/commit/6c6f9a80f9873f5989453c7800a355724cb61fff))
    - Added gzip error handling ([`6fb111c`](https://github.com/candlecorp/wick/commit/6fb111cc0068ca5a4709ef274b046c0b590eee08))
    - Added type imports ([`17c9058`](https://github.com/candlecorp/wick/commit/17c9058b98935fa8ed29dbc27b899c9e3244eb67))
</details>

## v0.4.0 (2023-09-14)

<csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/>
<csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/>
<csr-id-85ba75c8696fb8197c70b891f31724a3b069d59c/>
<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/>
<csr-id-a576880fa97834d9f89cfd7db4a42598b24fc02c/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/>

### Chore

 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-85ba75c8696fb8197c70b891f31724a3b069d59c/> formatting
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### New Features

 - <csr-id-7b60a70188be0c9be39138accee9329a810fc1e5/> implemented config cache
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/> added asset flags, fixed relative volumes, fixed manifest locations
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports

### Bug Fixes

 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-46d29109dc6502ea826236cf5438c54e02674d04/> fixed package push trying to retrieve remote assets
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-28c625552c460ac5c337efad3b0d621c9ec593cc/> don't fetch lazy assets
 - <csr-id-c1bb1d409adbf77c59da9e3241fa23d90cc39c8e/> added config fetch_all to fetch everything & made the default lazy
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-a576880fa97834d9f89cfd7db4a42598b24fc02c/> moved wick bin files to root
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/> removed double parse and added cheaper function to check OCI references

## v0.3.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/>
<csr-id-33b83d42f7a83e6ea81805f0ec0745654d12683f/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### New Features

 - <csr-id-7b60a70188be0c9be39138accee9329a810fc1e5/> implemented config cache
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/> added asset flags, fixed relative volumes, fixed manifest locations
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports

### Bug Fixes

 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-46d29109dc6502ea826236cf5438c54e02674d04/> fixed package push trying to retrieve remote assets
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-28c625552c460ac5c337efad3b0d621c9ec593cc/> don't fetch lazy assets
 - <csr-id-c1bb1d409adbf77c59da9e3241fa23d90cc39c8e/> added config fetch_all to fetch everything & made the default lazy
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-33b83d42f7a83e6ea81805f0ec0745654d12683f/> moved wick bin files to root
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-732db25382951c1cb5c245af35dd3fcbf2677a71/> removed double parse and added cheaper function to check OCI references

## v0.3.0 (2023-04-18)

<csr-id-7361b149ca108904341364426e1509105913f31f/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/>
<csr-id-7e2538202a03999c2b5781d7658b72118dce9446/>
<csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/>

### Chore

 - <csr-id-7361b149ca108904341364426e1509105913f31f/> release
   flow-component, flow-expression-parser, flow-graph, wick-asset-reference, wick-component, wick-config, wick-oci-utils
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses

### Refactor

 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils

### Test

 - <csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/> added registry tests, invoke tests, v1 tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 14 calendar days.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`7361b14`](https://github.com/candlecorp/wick/commit/7361b149ca108904341364426e1509105913f31f))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Added registry tests, invoke tests, v1 tests ([`3802bf9`](https://github.com/candlecorp/wick/commit/3802bf93746725527d5dfa80f3c65d3314d4122c))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
</details>

