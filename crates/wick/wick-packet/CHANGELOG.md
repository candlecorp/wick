# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.15.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-599514816356f7fab3b2122156092166f7815427/>
<csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/>
<csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/>
<csr-id-316111ac52d22365d060f573a456975de33b9115/>
<csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/>
<csr-id-695ae2be47c3da52b1690fe3b16282fb800a28f9/>
<csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/>
<csr-id-ff8b81dc1be6ff70237aaea1bc501b623f7c14d1/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings

### Chore

 - <csr-id-e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa/> generated changelogs

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-72a2fb3af224ff0b674c8e75a8c6e94070c181a7/> added packet assertions to wick test cases
 - <csr-id-bff97fe93ab537c2549893a33c8faa147dad0842/> added deep invocation, refactored runtime/engine names
 - <csr-id-517b96da7ba93357229b7c1725ecb3331120c636/> decoupled telemetry from log output
 - <csr-id-8fdef58ea207acb9ecb853c2c4934fe6daab39dd/> reorganized tracing span relationships
 - <csr-id-6d1949b2bc1012e9314b6e2e0637ac2225c87614/> improved type coercion, added mssql tests
 - <csr-id-32fcf67e9ba9c50695a5ee11e50b6674c5fdde96/> added wasi test component, added wick-operation proc_macros and adapter macros
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65/> rounded out preliminary support for switch substreams
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-a4160219ac2ba43cee39d31721eaf2821cd7906b/> made Base64Bytes the primary bytes struct, updated liquid_json
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-703988e288b32a1dc7f3d9dee232f4b4c79cc1cc/> made CLI parsing of arguments slightly smarter
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-a4dfea5a6d76b3f8d6df83758ac8bff9f5e744e7/> made wick test output more intuitive, updated rust template
 - <csr-id-85abe5adc703a9190b82dd78f58acdfe9920e3fe/> added better packet output in debugging mode
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-ba94e4dd43a85bb0dd79953f92b5a053e1536e62/> added op config to http client operations, added builders for config types
 - <csr-id-33c82afccdbcb4d7cda43e0ae880381501668478/> propagated seed to component context
 - <csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/> added context for wasm components
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-302612d5322fcc211b1ab7a05969c6de4bca7d7e/> added switch/case operation
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/> add Base64Bytes to wick-packet
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component
 - <csr-id-16940c8908ef9a463c227d8e8fdd5c1ad6bfc379/> added static router

### Bug Fixes

 - <csr-id-978690e2d05b2cae05991d273876a28f845abbb5/> fixed check in tests that resulted in duplicate DONE packets being sent
 - <csr-id-b5fbe25d31673d4e8676883cdeee7166a5538da5/> ensured missing values for optional fields do not throw an error
 - <csr-id-7c7d77df7f892e95b5dfde923c8e427078e6896b/> added more descriptive error and a continue on unknown port in streammap
 - <csr-id-516a395842bf80d81f17db30727ee2b5be69256f/> fixed ignored --filter argument on wick test
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-5f59bb11179ee19f49c82159e3b34f3abfe1c5ab/> fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-d3b4b02214d01cdc338cfb88a22f904bbb719134/> handled unwraps that led to in-wasm panics
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root

### Refactor

 - <csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/> renamed transaction to executioncontext in interpreter
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle
 - <csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/> updated rust-analyzer settings to be in line with CI checks, fixed lint errors
 - <csr-id-695ae2be47c3da52b1690fe3b16282fb800a28f9/> centralized date parsing logic into wick-packet
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation
 - <csr-id-ff8b81dc1be6ff70237aaea1bc501b623f7c14d1/> merged PacketStream into Invocation for invocations

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 58 commits contributed to the release over the course of 128 calendar days.
 - 131 days passed between releases.
 - 56 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 7 unique issues were worked on: [#232](https://github.com/candlecorp/wick/issues/232), [#278](https://github.com/candlecorp/wick/issues/278), [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#341](https://github.com/candlecorp/wick/issues/341), [#375](https://github.com/candlecorp/wick/issues/375), [#399](https://github.com/candlecorp/wick/issues/399)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#278](https://github.com/candlecorp/wick/issues/278)**
    - Handled unwraps that led to in-wasm panics ([`d3b4b02`](https://github.com/candlecorp/wick/commit/d3b4b02214d01cdc338cfb88a22f904bbb719134))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#341](https://github.com/candlecorp/wick/issues/341)**
    - Added ctx.inherent.timestamp, improved error message output ([`efe6055`](https://github.com/candlecorp/wick/commit/efe605510b846d2556f6060ba710fa154bdca7c4))
 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **[#399](https://github.com/candlecorp/wick/issues/399)**
    - Better http client substream support. ([`744f1ac`](https://github.com/candlecorp/wick/commit/744f1ac3d5fa8c28e8e0a1e80d7f5e49839c0c43))
 * **Uncategorized**
    - Generated changelogs ([`e1d6c05`](https://github.com/candlecorp/wick/commit/e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa))
    - Added packet assertions to wick test cases ([`72a2fb3`](https://github.com/candlecorp/wick/commit/72a2fb3af224ff0b674c8e75a8c6e94070c181a7))
    - Added deep invocation, refactored runtime/engine names ([`bff97fe`](https://github.com/candlecorp/wick/commit/bff97fe93ab537c2549893a33c8faa147dad0842))
    - Decoupled telemetry from log output ([`517b96d`](https://github.com/candlecorp/wick/commit/517b96da7ba93357229b7c1725ecb3331120c636))
    - Renamed transaction to executioncontext in interpreter ([`37030ca`](https://github.com/candlecorp/wick/commit/37030caa9d8930774f6cac2f0b921d6f7d793941))
    - Reorganized tracing span relationships ([`8fdef58`](https://github.com/candlecorp/wick/commit/8fdef58ea207acb9ecb853c2c4934fe6daab39dd))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Improved type coercion, added mssql tests ([`6d1949b`](https://github.com/candlecorp/wick/commit/6d1949b2bc1012e9314b6e2e0637ac2225c87614))
    - Added wasi test component, added wick-operation proc_macros and adapter macros ([`32fcf67`](https://github.com/candlecorp/wick/commit/32fcf67e9ba9c50695a5ee11e50b6674c5fdde96))
    - Fixed check in tests that resulted in duplicate DONE packets being sent ([`978690e`](https://github.com/candlecorp/wick/commit/978690e2d05b2cae05991d273876a28f845abbb5))
    - Ensured missing values for optional fields do not throw an error ([`b5fbe25`](https://github.com/candlecorp/wick/commit/b5fbe25d31673d4e8676883cdeee7166a5538da5))
    - Added inheritance/delegation to composite components, reorganized test files ([`e46db5f`](https://github.com/candlecorp/wick/commit/e46db5f2138254c227a2c39a3821074b77cf0166))
    - Added more descriptive error and a continue on unknown port in streammap ([`7c7d77d`](https://github.com/candlecorp/wick/commit/7c7d77df7f892e95b5dfde923c8e427078e6896b))
    - Adjusted logging, interpreter execution lifecycle ([`316111a`](https://github.com/candlecorp/wick/commit/316111ac52d22365d060f573a456975de33b9115))
    - Rounded out preliminary support for switch substreams ([`a8232d0`](https://github.com/candlecorp/wick/commit/a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65))
    - Fixed ignored --filter argument on wick test ([`516a395`](https://github.com/candlecorp/wick/commit/516a395842bf80d81f17db30727ee2b5be69256f))
    - Made Base64Bytes the primary bytes struct, updated liquid_json ([`a416021`](https://github.com/candlecorp/wick/commit/a4160219ac2ba43cee39d31721eaf2821cd7906b))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Updated rust-analyzer settings to be in line with CI checks, fixed lint errors ([`f5c8df4`](https://github.com/candlecorp/wick/commit/f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7))
    - Added on_error & transaction support to ms sql server SQL implementation ([`d85d6f5`](https://github.com/candlecorp/wick/commit/d85d6f568d4548036c1af61e515c3fc187be6a6e))
    - Fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb ([`5f59bb1`](https://github.com/candlecorp/wick/commit/5f59bb11179ee19f49c82159e3b34f3abfe1c5ab))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Made CLI parsing of arguments slightly smarter ([`703988e`](https://github.com/candlecorp/wick/commit/703988e288b32a1dc7f3d9dee232f4b4c79cc1cc))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Updated to rust 1.69.0, fixed associated warnings ([`e561fd6`](https://github.com/candlecorp/wick/commit/e561fd668afb1e1af3639c472a893b7fcfe2bf54))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Made wick test output more intuitive, updated rust template ([`a4dfea5`](https://github.com/candlecorp/wick/commit/a4dfea5a6d76b3f8d6df83758ac8bff9f5e744e7))
    - Centralized date parsing logic into wick-packet ([`695ae2b`](https://github.com/candlecorp/wick/commit/695ae2be47c3da52b1690fe3b16282fb800a28f9))
    - Added better packet output in debugging mode ([`85abe5a`](https://github.com/candlecorp/wick/commit/85abe5adc703a9190b82dd78f58acdfe9920e3fe))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Merged PacketStream into Invocation for invocations ([`ff8b81d`](https://github.com/candlecorp/wick/commit/ff8b81dc1be6ff70237aaea1bc501b623f7c14d1))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Added restapi router ([`58045d0`](https://github.com/candlecorp/wick/commit/58045d0fe75f519b84ebd45f3b1493e55fd4b282))
    - Added op config to http client operations, added builders for config types ([`ba94e4d`](https://github.com/candlecorp/wick/commit/ba94e4dd43a85bb0dd79953f92b5a053e1536e62))
    - Propagated seed to component context ([`33c82af`](https://github.com/candlecorp/wick/commit/33c82afccdbcb4d7cda43e0ae880381501668478))
    - Added context for wasm components ([`27c1fba`](https://github.com/candlecorp/wick/commit/27c1fba1d6af314e3b5f317178426331acc4b071))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added switch/case operation ([`302612d`](https://github.com/candlecorp/wick/commit/302612d5322fcc211b1ab7a05969c6de4bca7d7e))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Add Base64Bytes to wick-packet ([`399c5d5`](https://github.com/candlecorp/wick/commit/399c5d518b0a291dba63fb3f69337af2911d1776))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
    - Added static router ([`16940c8`](https://github.com/candlecorp/wick/commit/16940c8908ef9a463c227d8e8fdd5c1ad6bfc379))
</details>

## v0.15.0 (2023-04-18)

<csr-id-35047c3a741b00d88c4abc2ed3749af040a83671/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>

### Chore

 - <csr-id-35047c3a741b00d88c4abc2ed3749af040a83671/> release wick-xdg, wick-logger, asset-container, derive-asset-container, performance-mark, tap-harness, wick-interface-types, wick-packet

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 19 calendar days.
 - 26 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release wick-xdg, wick-logger, asset-container, derive-asset-container, performance-mark, tap-harness, wick-interface-types, wick-packet ([`35047c3`](https://github.com/candlecorp/wick/commit/35047c3a741b00d88c4abc2ed3749af040a83671))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
</details>

## v0.14.0 (2023-03-23)

<csr-id-501d6056a5ff2d06290f88f73885c6c12afd77e9/>

### Chore

 - <csr-id-501d6056a5ff2d06290f88f73885c6c12afd77e9/> Release

### New Features

 - <csr-id-ade73755500573d2dec3ebf0e7113f73fa238549/> added pretty JSON output to wick invoke commands

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`501d605`](https://github.com/candlecorp/wick/commit/501d6056a5ff2d06290f88f73885c6c12afd77e9))
    - Added pretty JSON output to wick invoke commands ([`ade7375`](https://github.com/candlecorp/wick/commit/ade73755500573d2dec3ebf0e7113f73fa238549))
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

### Bug Fixes

 - <csr-id-5c807f221fbb2eefaedaa899f82da3e8f2600388/> fixed broken tests
 - <csr-id-63323a7127eb1d9d9e27af2c3771945d7d16504e/> isolated wasmrs-guest references to wasm-only conditionally compiled locations.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 7 calendar days.
 - 8 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#153](https://github.com/candlecorp/wick/issues/153)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#153](https://github.com/candlecorp/wick/issues/153)**
    - Isolated wasmrs-guest references to wasm-only conditionally compiled locations. ([`63323a7`](https://github.com/candlecorp/wick/commit/63323a7127eb1d9d9e27af2c3771945d7d16504e))
 * **Uncategorized**
    - Release ([`f229d8e`](https://github.com/candlecorp/wick/commit/f229d8ee9dbb1c051d18b911bb4ef868b968ea14))
    - Fixed broken tests ([`5c807f2`](https://github.com/candlecorp/wick/commit/5c807f221fbb2eefaedaa899f82da3e8f2600388))
    - Substreams/bracketing + codegen improvements ([`39fb923`](https://github.com/candlecorp/wick/commit/39fb923c30ec819bcbe665ef4fad569eebdfe194))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
</details>

## v0.12.0 (2023-03-15)

<csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/>

### Chore

 - <csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/> renamed existing wafl references

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 4 calendar days.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed existing wafl references ([`3a42e63`](https://github.com/candlecorp/wick/commit/3a42e6388e3561103412ca3e47db8b5feb5ef3a9))
    - Shoring up tests. fixed error propagation and hung txs stemming from timeouts ([`46310b9`](https://github.com/candlecorp/wick/commit/46310b98b6933c5a6d84c32863391bb482af5ac3))
    - Renamed wick-config-component to wick-config, added app config, restructured triggers, added trigger test component ([`24ef43f`](https://github.com/candlecorp/wick/commit/24ef43f7fc978c1f33f27a1e90f9971abdeb9b11))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

