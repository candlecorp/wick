# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.23.0 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-6cbc8b53e1f68fa5336220261fc80f0256601133/>
<csr-id-a0d92a6462f139a598be39decd633ceb7a956113/>
<csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/>
<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-599514816356f7fab3b2122156092166f7815427/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-cf597555a592d7d05b4541395d81e0eed5e35a10/>
<csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/>
<csr-id-6aecefa7d7fe4e806b239cf9cadb914837c10dbe/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-316111ac52d22365d060f573a456975de33b9115/>
<csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/>
<csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/>
<csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/>
<csr-id-6974470aa8a5fa58a0a4de07811a5da9bec6c1cc/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-6cbc8b53e1f68fa5336220261fc80f0256601133/> added experimental settings section, removed incomplete example
 - <csr-id-a0d92a6462f139a598be39decd633ceb7a956113/> disabled some default features on wasmtime
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases

### Chore

 - <csr-id-e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa/> generated changelogs

### Documentation

 - <csr-id-10672c5db34d10e50869b2c14977f9235761cabd/> updated config codegen, refactored config for clarity, fixed template

### New Features

<csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/>
<csr-id-ba2015ddf2d24324c311fa681a39c4a65ac886bc/>
<csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/>
<csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/>
<csr-id-f575c65c9579db77ae053c37ae2eff02716136ab/>
<csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/>
<csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/>
<csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/>
<csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/>
<csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/>
<csr-id-302612d5322fcc211b1ab7a05969c6de4bca7d7e/>
<csr-id-0f05d770d08d86fc256154739b62ff089e26b503/>
<csr-id-027392a9514ba4846e068b21476e980ea53bee1d/>
<csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/>
<csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/>
<csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/>
<csr-id-cbd6515303db5bb5fb9383116f0ee69a90e4c537/>
<csr-id-16940c8908ef9a463c227d8e8fdd5c1ad6bfc379/>
<csr-id-4c86477ce3176b546e06dc0e9db969921babe3d6/>

 - <csr-id-bff97fe93ab537c2549893a33c8faa147dad0842/> added deep invocation, refactored runtime/engine names
 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-7b60a70188be0c9be39138accee9329a810fc1e5/> implemented config cache
 - <csr-id-70f0fd07ac70ae4fd1bb1734b306266f14f3af3c/> made buffer_size configurable
 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-8fdef58ea207acb9ecb853c2c4934fe6daab39dd/> reorganized tracing span relationships
 - <csr-id-6d1949b2bc1012e9314b6e2e0637ac2225c87614/> improved type coercion, added mssql tests
 - <csr-id-71ba0230aadd9c31d05ebef3478247dbf200fa1d/> brought postgres and sqlite up-to-date with mssql
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-e5ed32378e0fd61c8bb1560027d252c0c93059a1/> added wick config dotviz, made interpreter tolerant of unused ports
 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-b679aad2e505e2e4b15794dc4decc98c51aee077/> added v1 wasm signatures, bumped wasmrs, enabled module cache
 - <csr-id-3eb6ac3742b7cebaff7cf5dbf3e552cc6cd784f3/> reused wasmtime engine from runtime, updated wasm parser
 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65/> rounded out preliminary support for switch substreams
 - <csr-id-a4160219ac2ba43cee39d31721eaf2821cd7906b/> made Base64Bytes the primary bytes struct, updated liquid_json
 - <csr-id-1528f18c896c16ba798d37dcca5e017beecfd7c2/> added openapi spec generation
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-64e30fbb7e64e7f744190ebcbab107b4916a24e1/> better discriminated HTTP errors, removed error output from 500 responses
 - <csr-id-bd8af683437d46ed7281fd8cd806efe22ffa0f6f/> added quote-delimeted paths to field syntax, made rest router return errors on error packets
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-85abe5adc703a9190b82dd78f58acdfe9920e3fe/> added better packet output in debugging mode
 - <csr-id-103c9d8e67fff895d02c10597faedfe8b72d1eab/> added fallback option for static http
   * feat: added fallback option for static http
* fix: fix clippy error
* refactor: cleaned up code for style
* fix: corrected documentation
* fix: remove async from response function

### Bug Fixes

 - <csr-id-83e49dcf595a23bf120d62a770c982e81e0b0e99/> made cron span less verbose
 - <csr-id-7d960422708edf6d59cb3c74ffac701fb5e1bd3b/> fixed time trigger and made hard failures cancel the scheduler
 - <csr-id-3239a4453868d04ea32ace557cc14ca75a3045e8/> reused existing imports in triggers and http routers
 - <csr-id-3208691ffb824e9f83d9845ae274c9b60bb8d4fa/> converted all level spans to info_spans
 - <csr-id-3b684528061d9c6a61ca4455415b96bfab0542dd/> fixed hanging tx for raw router components, removed bad debug line
 - <csr-id-3f9e3ea21b9e6cbe6a6635681d0ad4ccec7f6642/> fixed path for rest-router example and tests
 - <csr-id-d901966927c3eec44270bbd2cd5d84baaa1f3462/> fixed relative volumes again
 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-3108cf583cf49a93b706be93ce87c47f77633727/> corrected openapi path + replaced name with id in rest router config
 - <csr-id-5589b1e55e25352fa5c26902278555c75231a05d/> fixed broken test
 - <csr-id-4a0faa4fb54861f6c01a9809a15217f89a65f6cd/> fixed race condition in middleware, turned muffled errors into ISEs
 - <csr-id-3b0dba65afb39d0b67c22c62ab8bc407052dedce/> fixed race condition in request middleware
 - <csr-id-e107d7cc2fb3d36925fe8af471b164c07ec3e15d/> fixed broken cache path, fixed unrendered Volume configuraton
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-d1a96a3a67f1a92f4966ffded5ac99b29b07f172/> fixed re-render of configuration in trigger context
 - <csr-id-91add7617883319ea1f485b02d8ee51738fb90e5/> fixed array types on query strings, not parsing query params with single param
 - <csr-id-2543554aac59ec07494aff486e896719f92cb810/> rest_router: stopped input port from being pushed to on GET requests
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-5f59bb11179ee19f49c82159e3b34f3abfe1c5ab/> fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb
 - <csr-id-4e3bae9b2e195ad14ebcc495f0efc90b583e2381/> changed an unmatched path from an 501 to a 404
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-44f79725516fcc0d32880e2f8ff9dd1107433511/> fixed incorrect field parsing of empty query strings
 - <csr-id-efdc1f0082b5cb73fa060d83e84d4bdb13f819a3/> fixed error on implicit db output:object, improved error details, renamed examples
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-c0ab15b0cf854a4ae8047c9f00d6da85febe0db2/> updated trace configuration, added jaeger endpoint to config.yaml settings
 - <csr-id-d8d8a5cfc84964d59b3839cf3248c764de15e3f1/> fixed trigger import loading, removed aggressive http panic
 - <csr-id-9053e403a32eff847be6d43e623a464fa0377395/> fixed sql bound arguments and postgres encodings
 - <csr-id-12dc502a2b6bc62a9ca01176a27da60c0407efd4/> fixed cause of content-length mismatch in http trigger

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-cf597555a592d7d05b4541395d81e0eed5e35a10/> making global cache default for wick run
 - <csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/> renamed transaction to executioncontext in interpreter
 - <csr-id-6aecefa7d7fe4e806b239cf9cadb914837c10dbe/> removed experimental block, changed expose to extends
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle
 - <csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/> updated rust-analyzer settings to be in line with CI checks, fixed lint errors
 - <csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/> removed conflicting timeouts in favor of per-op timeouts
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation

### Test

 - <csr-id-6974470aa8a5fa58a0a4de07811a5da9bec6c1cc/> made time trigger test more reliable

### New Features (BREAKING)

 - <csr-id-34e1484443de014ebe010063640f937e528df10a/> changed pre-request middleware to one output union vs a request/response race

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 104 commits contributed to the release over the course of 130 calendar days.
 - 131 days passed between releases.
 - 103 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 6 unique issues were worked on: [#232](https://github.com/candlecorp/wick/issues/232), [#254](https://github.com/candlecorp/wick/issues/254), [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#405](https://github.com/candlecorp/wick/issues/405), [#416](https://github.com/candlecorp/wick/issues/416)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#254](https://github.com/candlecorp/wick/issues/254)**
    - Added fallback option for static http ([`103c9d8`](https://github.com/candlecorp/wick/commit/103c9d8e67fff895d02c10597faedfe8b72d1eab))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#405](https://github.com/candlecorp/wick/issues/405)**
    - Fixed "refusing to overwrite ..." errors on application runs. ([`a10242d`](https://github.com/candlecorp/wick/commit/a10242d4786cfa199eaf61289b9da99d09c114a7))
 * **[#416](https://github.com/candlecorp/wick/issues/416)**
    - Made cron span less verbose ([`83e49dc`](https://github.com/candlecorp/wick/commit/83e49dcf595a23bf120d62a770c982e81e0b0e99))
 * **Uncategorized**
    - Generated changelogs ([`e1d6c05`](https://github.com/candlecorp/wick/commit/e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa))
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Made time trigger test more reliable ([`6974470`](https://github.com/candlecorp/wick/commit/6974470aa8a5fa58a0a4de07811a5da9bec6c1cc))
    - Added deep invocation, refactored runtime/engine names ([`bff97fe`](https://github.com/candlecorp/wick/commit/bff97fe93ab537c2549893a33c8faa147dad0842))
    - Making global cache default for wick run ([`cf59755`](https://github.com/candlecorp/wick/commit/cf597555a592d7d05b4541395d81e0eed5e35a10))
    - Added wick audit & lockdown config ([`ddf1008`](https://github.com/candlecorp/wick/commit/ddf1008983c1f4a880a42ac4c29c0f60bc619cf3))
    - Fixed time trigger and made hard failures cancel the scheduler ([`7d96042`](https://github.com/candlecorp/wick/commit/7d960422708edf6d59cb3c74ffac701fb5e1bd3b))
    - Reused existing imports in triggers and http routers ([`3239a44`](https://github.com/candlecorp/wick/commit/3239a4453868d04ea32ace557cc14ca75a3045e8))
    - Implemented config cache ([`7b60a70`](https://github.com/candlecorp/wick/commit/7b60a70188be0c9be39138accee9329a810fc1e5))
    - Made buffer_size configurable ([`70f0fd0`](https://github.com/candlecorp/wick/commit/70f0fd07ac70ae4fd1bb1734b306266f14f3af3c))
    - Support provides/requires relationship in composite components ([`8ceae1a`](https://github.com/candlecorp/wick/commit/8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d))
    - Converted all level spans to info_spans ([`3208691`](https://github.com/candlecorp/wick/commit/3208691ffb824e9f83d9845ae274c9b60bb8d4fa))
    - Renamed transaction to executioncontext in interpreter ([`37030ca`](https://github.com/candlecorp/wick/commit/37030caa9d8930774f6cac2f0b921d6f7d793941))
    - Reorganized tracing span relationships ([`8fdef58`](https://github.com/candlecorp/wick/commit/8fdef58ea207acb9ecb853c2c4934fe6daab39dd))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Fixed hanging tx for raw router components, removed bad debug line ([`3b68452`](https://github.com/candlecorp/wick/commit/3b684528061d9c6a61ca4455415b96bfab0542dd))
    - Fixed path for rest-router example and tests ([`3f9e3ea`](https://github.com/candlecorp/wick/commit/3f9e3ea21b9e6cbe6a6635681d0ad4ccec7f6642))
    - Fixed relative volumes again ([`d901966`](https://github.com/candlecorp/wick/commit/d901966927c3eec44270bbd2cd5d84baaa1f3462))
    - Improved type coercion, added mssql tests ([`6d1949b`](https://github.com/candlecorp/wick/commit/6d1949b2bc1012e9314b6e2e0637ac2225c87614))
    - Brought postgres and sqlite up-to-date with mssql ([`71ba023`](https://github.com/candlecorp/wick/commit/71ba0230aadd9c31d05ebef3478247dbf200fa1d))
    - Added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax ([`b0b9cd2`](https://github.com/candlecorp/wick/commit/b0b9cd20f748ffe1956ad2501fe23991fededf13))
    - Re-added exposing volumes to WASI components ([`ce9d202`](https://github.com/candlecorp/wick/commit/ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85))
    - Added `wick config expand` ([`33ea9cd`](https://github.com/candlecorp/wick/commit/33ea9cd5fff9a85398e7fc15661cb9401a085c18))
    - Added wick config dotviz, made interpreter tolerant of unused ports ([`e5ed323`](https://github.com/candlecorp/wick/commit/e5ed32378e0fd61c8bb1560027d252c0c93059a1))
    - Added flow sequences, enhanced port inference ([`2a5cf0c`](https://github.com/candlecorp/wick/commit/2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695))
    - Added more tests for #378, fixed fields being requide by default from config ([`ae1400c`](https://github.com/candlecorp/wick/commit/ae1400caa092433bec0f66c04bd6e0efea30d173))
    - Removed experimental block, changed expose to extends ([`6aecefa`](https://github.com/candlecorp/wick/commit/6aecefa7d7fe4e806b239cf9cadb914837c10dbe))
    - Added experimental settings section, removed incomplete example ([`6cbc8b5`](https://github.com/candlecorp/wick/commit/6cbc8b53e1f68fa5336220261fc80f0256601133))
    - Added inheritance/delegation to composite components, reorganized test files ([`e46db5f`](https://github.com/candlecorp/wick/commit/e46db5f2138254c227a2c39a3821074b77cf0166))
    - Corrected openapi path + replaced name with id in rest router config ([`3108cf5`](https://github.com/candlecorp/wick/commit/3108cf583cf49a93b706be93ce87c47f77633727))
    - Eliminated fetching of bytes before checking cache ([`586ace0`](https://github.com/candlecorp/wick/commit/586ace0978ca8adf58bf4d1fa5ed392015297c21))
    - Disabled some default features on wasmtime ([`a0d92a6`](https://github.com/candlecorp/wick/commit/a0d92a6462f139a598be39decd633ceb7a956113))
    - Added v1 wasm signatures, bumped wasmrs, enabled module cache ([`b679aad`](https://github.com/candlecorp/wick/commit/b679aad2e505e2e4b15794dc4decc98c51aee077))
    - Reused wasmtime engine from runtime, updated wasm parser ([`3eb6ac3`](https://github.com/candlecorp/wick/commit/3eb6ac3742b7cebaff7cf5dbf3e552cc6cd784f3))
    - Fixed broken test ([`5589b1e`](https://github.com/candlecorp/wick/commit/5589b1e55e25352fa5c26902278555c75231a05d))
    - Changed pre-request middleware to one output union vs a request/response race ([`34e1484`](https://github.com/candlecorp/wick/commit/34e1484443de014ebe010063640f937e528df10a))
    - Added unions to type definitions ([`222cc7f`](https://github.com/candlecorp/wick/commit/222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f))
    - Adjusted logging, interpreter execution lifecycle ([`316111a`](https://github.com/candlecorp/wick/commit/316111ac52d22365d060f573a456975de33b9115))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Rounded out preliminary support for switch substreams ([`a8232d0`](https://github.com/candlecorp/wick/commit/a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65))
    - Made Base64Bytes the primary bytes struct, updated liquid_json ([`a416021`](https://github.com/candlecorp/wick/commit/a4160219ac2ba43cee39d31721eaf2821cd7906b))
    - Fixed race condition in middleware, turned muffled errors into ISEs ([`4a0faa4`](https://github.com/candlecorp/wick/commit/4a0faa4fb54861f6c01a9809a15217f89a65f6cd))
    - Fixed race condition in request middleware ([`3b0dba6`](https://github.com/candlecorp/wick/commit/3b0dba65afb39d0b67c22c62ab8bc407052dedce))
    - Fixed broken cache path, fixed unrendered Volume configuraton ([`e107d7c`](https://github.com/candlecorp/wick/commit/e107d7cc2fb3d36925fe8af471b164c07ec3e15d))
    - Added openapi spec generation ([`1528f18`](https://github.com/candlecorp/wick/commit/1528f18c896c16ba798d37dcca5e017beecfd7c2))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - Fixed re-render of configuration in trigger context ([`d1a96a3`](https://github.com/candlecorp/wick/commit/d1a96a3a67f1a92f4966ffded5ac99b29b07f172))
    - Fixed array types on query strings, not parsing query params with single param ([`91add76`](https://github.com/candlecorp/wick/commit/91add7617883319ea1f485b02d8ee51738fb90e5))
    - Rest_router: stopped input port from being pushed to on GET requests ([`2543554`](https://github.com/candlecorp/wick/commit/2543554aac59ec07494aff486e896719f92cb810))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Updated rust-analyzer settings to be in line with CI checks, fixed lint errors ([`f5c8df4`](https://github.com/candlecorp/wick/commit/f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7))
    - Removed conflicting timeouts in favor of per-op timeouts ([`888814b`](https://github.com/candlecorp/wick/commit/888814bb24d3d4dd4b460af2616a72814f2bd7a1))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Added on_error & transaction support to ms sql server SQL implementation ([`d85d6f5`](https://github.com/candlecorp/wick/commit/d85d6f568d4548036c1af61e515c3fc187be6a6e))
    - Fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb ([`5f59bb1`](https://github.com/candlecorp/wick/commit/5f59bb11179ee19f49c82159e3b34f3abfe1c5ab))
    - Better discriminated HTTP errors, removed error output from 500 responses ([`64e30fb`](https://github.com/candlecorp/wick/commit/64e30fbb7e64e7f744190ebcbab107b4916a24e1))
    - Changed an unmatched path from an 501 to a 404 ([`4e3bae9`](https://github.com/candlecorp/wick/commit/4e3bae9b2e195ad14ebcc495f0efc90b583e2381))
    - Added quote-delimeted paths to field syntax, made rest router return errors on error packets ([`bd8af68`](https://github.com/candlecorp/wick/commit/bd8af683437d46ed7281fd8cd806efe22ffa0f6f))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Updated config codegen, refactored config for clarity, fixed template ([`10672c5`](https://github.com/candlecorp/wick/commit/10672c5db34d10e50869b2c14977f9235761cabd))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Fixed incorrect field parsing of empty query strings ([`44f7972`](https://github.com/candlecorp/wick/commit/44f79725516fcc0d32880e2f8ff9dd1107433511))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Fixed error on implicit db output:object, improved error details, renamed examples ([`efdc1f0`](https://github.com/candlecorp/wick/commit/efdc1f0082b5cb73fa060d83e84d4bdb13f819a3))
    - Added better packet output in debugging mode ([`85abe5a`](https://github.com/candlecorp/wick/commit/85abe5adc703a9190b82dd78f58acdfe9920e3fe))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Updated trace configuration, added jaeger endpoint to config.yaml settings ([`c0ab15b`](https://github.com/candlecorp/wick/commit/c0ab15b0cf854a4ae8047c9f00d6da85febe0db2))
    - Fixed trigger import loading, removed aggressive http panic ([`d8d8a5c`](https://github.com/candlecorp/wick/commit/d8d8a5cfc84964d59b3839cf3248c764de15e3f1))
    - Added settings file, wick reg login, & wick reg push --latest ([`63858e1`](https://github.com/candlecorp/wick/commit/63858e1bc6673b61d50fa8f66dc4378369850910))
    - Added azure-sql support ([`ba2015d`](https://github.com/candlecorp/wick/commit/ba2015ddf2d24324c311fa681a39c4a65ac886bc))
    - Fixed sql bound arguments and postgres encodings ([`9053e40`](https://github.com/candlecorp/wick/commit/9053e403a32eff847be6d43e623a464fa0377395))
    - Added restapi router ([`58045d0`](https://github.com/candlecorp/wick/commit/58045d0fe75f519b84ebd45f3b1493e55fd4b282))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Added http trigger logging ([`f575c65`](https://github.com/candlecorp/wick/commit/f575c65c9579db77ae053c37ae2eff02716136ab))
    - Added proper type defs into config, closes #200. Fixed #228, #227 ([`49a53de`](https://github.com/candlecorp/wick/commit/49a53de6cb6631e2dc1f1e633d1c29d0510383cb))
    - Added context for wasm components ([`27c1fba`](https://github.com/candlecorp/wick/commit/27c1fba1d6af314e3b5f317178426331acc4b071))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added asset flags, fixed relative volumes, fixed manifest locations ([`3dd4cdb`](https://github.com/candlecorp/wick/commit/3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08))
    - Added switch/case operation ([`302612d`](https://github.com/candlecorp/wick/commit/302612d5322fcc211b1ab7a05969c6de4bca7d7e))
    - Added sub-flow operatiions ([`0f05d77`](https://github.com/candlecorp/wick/commit/0f05d770d08d86fc256154739b62ff089e26b503))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Add Base64Bytes to wick-packet ([`399c5d5`](https://github.com/candlecorp/wick/commit/399c5d518b0a291dba63fb3f69337af2911d1776))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
    - Added type imports ([`17c9058`](https://github.com/candlecorp/wick/commit/17c9058b98935fa8ed29dbc27b899c9e3244eb67))
    - Added reverse proxy router ([`cbd6515`](https://github.com/candlecorp/wick/commit/cbd6515303db5bb5fb9383116f0ee69a90e4c537))
    - Fixed cause of content-length mismatch in http trigger ([`12dc502`](https://github.com/candlecorp/wick/commit/12dc502a2b6bc62a9ca01176a27da60c0407efd4))
    - Added static router ([`16940c8`](https://github.com/candlecorp/wick/commit/16940c8908ef9a463c227d8e8fdd5c1ad6bfc379))
    - Added URL resource, migrated sql component to it ([`4c86477`](https://github.com/candlecorp/wick/commit/4c86477ce3176b546e06dc0e9db969921babe3d6))
</details>

<csr-unknown>
 added settings file, wick reg login, & wick reg push –latest added azure-sql support added restapi router normalized accessor api for wick-config added http trigger logging added codec to HTTP server, added runtime constraints, ability to explicitly drop packets added proper type defs into config, closes #200. Fixed #228, #227 added context for wasm components added operation context added asset flags, fixed relative volumes, fixed manifest locations added switch/case operation added sub-flow operatiions added pluck & merge add Base64Bytes to wick-packet added http client component added type imports added reverse proxy router added static router added URL resource, migrated sql component to it<csr-unknown/>

## v0.22.0 (2023-04-19)

<csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/>
<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-406c10999648ca923fc8994b5835d11c823c19ce/>
<csr-id-88c97a7ddca56ace4e7aeacbc2dcc4d47a0b11d0/>
<csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-42ade875f501b69b80a09ff86a1be33ddee14ec3/>
<csr-id-7e2538202a03999c2b5781d7658b72118dce9446/>
<csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/>
<csr-id-890b9dd879e9d18c8e989989a01e73eb5a987b2f/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>
<csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/>
<csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/>

### Chore

 - <csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/> release wick-cli and rest of crates
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages
 - <csr-id-88c97a7ddca56ace4e7aeacbc2dcc4d47a0b11d0/> removed dead code
 - <csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/> renamed existing wafl references

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-154c09b8b1169cb92bbc35135ab516e42c51e5d0/> add bdd and time trigger
 - <csr-id-0ce9f5573b827fa5e5d7d8dd5bac102e890a66e1/> propagated the network timeout to the interpreter
 - <csr-id-b83485305d609f9f599ae4a3f0aa03d9e101fb5c/> centralized relative file resolution within wick-config
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger
 - <csr-id-8745221bb0e25332f85bebe2387bc10a440ed5ac/> added codegen based off component.yaml
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Bug Fixes

 - <csr-id-66089ef51f87994a6a2be3a31f365f2226b81830/> changed postgres component to generic sql component
 - <csr-id-46c3bd67a13e349280d16ce50c336a5415ef589c/> clippy style
 - <csr-id-1c58123f86ec95073b503790fe272b04003a05df/> adjusted default features on deps
 - <csr-id-16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc/> path resolution and missing wasm components in interpreter
 - <csr-id-5c807f221fbb2eefaedaa899f82da3e8f2600388/> fixed broken tests

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable
 - <csr-id-42ade875f501b69b80a09ff86a1be33ddee14ec3/> defer to Into implementation for packet->PacketStream
 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils
 - <csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/> centralized APIs around configuration yaml

### Test

 - <csr-id-890b9dd879e9d18c8e989989a01e73eb5a987b2f/> moved tests with native-component to separate rust project in test/integration
 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup
 - <csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/> added registry tests, invoke tests, v1 tests

### Refactor (BREAKING)

 - <csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/> removed "default" value substitution in favor of a future impl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 34 commits contributed to the release over the course of 39 calendar days.
 - 28 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Release wick-cli and rest of crates ([`1279be0`](https://github.com/candlecorp/wick/commit/1279be06f6cf8bc91641be7ab48d7941819c98fe))
    - Moved tests with native-component to separate rust project in test/integration ([`890b9dd`](https://github.com/candlecorp/wick/commit/890b9dd879e9d18c8e989989a01e73eb5a987b2f))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Changed postgres component to generic sql component ([`66089ef`](https://github.com/candlecorp/wick/commit/66089ef51f87994a6a2be3a31f365f2226b81830))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Defer to Into implementation for packet->PacketStream ([`42ade87`](https://github.com/candlecorp/wick/commit/42ade875f501b69b80a09ff86a1be33ddee14ec3))
    - Clippy style ([`46c3bd6`](https://github.com/candlecorp/wick/commit/46c3bd67a13e349280d16ce50c336a5415ef589c))
    - Add bdd and time trigger ([`154c09b`](https://github.com/candlecorp/wick/commit/154c09b8b1169cb92bbc35135ab516e42c51e5d0))
    - Adjusted default features on deps ([`1c58123`](https://github.com/candlecorp/wick/commit/1c58123f86ec95073b503790fe272b04003a05df))
    - Propagated the network timeout to the interpreter ([`0ce9f55`](https://github.com/candlecorp/wick/commit/0ce9f5573b827fa5e5d7d8dd5bac102e890a66e1))
    - Added registry tests, invoke tests, v1 tests ([`3802bf9`](https://github.com/candlecorp/wick/commit/3802bf93746725527d5dfa80f3c65d3314d4122c))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
    - Path resolution and missing wasm components in interpreter ([`16bb6b4`](https://github.com/candlecorp/wick/commit/16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc))
    - Centralized relative file resolution within wick-config ([`b834853`](https://github.com/candlecorp/wick/commit/b83485305d609f9f599ae4a3f0aa03d9e101fb5c))
    - Centralized APIs around configuration yaml ([`fd71df4`](https://github.com/candlecorp/wick/commit/fd71df4baaa3f856454624396eff9d9ee8c4473f))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
    - Fixed broken tests ([`5c807f2`](https://github.com/candlecorp/wick/commit/5c807f221fbb2eefaedaa899f82da3e8f2600388))
    - Removed "default" value substitution in favor of a future impl ([`c7b84da`](https://github.com/candlecorp/wick/commit/c7b84daacad21d9ba2c44123a6b0695db3b43528))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Removed dead code ([`88c97a7`](https://github.com/candlecorp/wick/commit/88c97a7ddca56ace4e7aeacbc2dcc4d47a0b11d0))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed existing wafl references ([`3a42e63`](https://github.com/candlecorp/wick/commit/3a42e6388e3561103412ca3e47db8b5feb5ef3a9))
    - Added codegen based off component.yaml ([`8745221`](https://github.com/candlecorp/wick/commit/8745221bb0e25332f85bebe2387bc10a440ed5ac))
    - Incorporated interface.json into component.yaml ([`97280ee`](https://github.com/candlecorp/wick/commit/97280ee71b361472dbb6ae32c77626b07c218554))
    - Shoring up tests. fixed error propagation and hung txs stemming from timeouts ([`46310b9`](https://github.com/candlecorp/wick/commit/46310b98b6933c5a6d84c32863391bb482af5ac3))
    - Renamed wick-config-component to wick-config, added app config, restructured triggers, added trigger test component ([`24ef43f`](https://github.com/candlecorp/wick/commit/24ef43f7fc978c1f33f27a1e90f9971abdeb9b11))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

