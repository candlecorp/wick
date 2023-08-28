# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.20.1 (2023-08-28)

<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/>
<csr-id-820e9ee4d0b4f0126183f071d56a422322e7a257/>
<csr-id-d807f943bc550df8a2cda4c246bbf765f1674065/>
<csr-id-3580951b5faa8ef279291e5a6f994d1c9e0785d6/>
<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-599514816356f7fab3b2122156092166f7815427/>
<csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/>
<csr-id-180746ab27766c9df21b334482bead0da8f0bfba/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/>
<csr-id-18a767a9d2e45c9efd1d3cbabe87b4450b78b255/>
<csr-id-316111ac52d22365d060f573a456975de33b9115/>
<csr-id-39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f/>
<csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/>
<csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/>
<csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/>
<csr-id-ff8b81dc1be6ff70237aaea1bc501b623f7c14d1/>

### Chore

 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-820e9ee4d0b4f0126183f071d56a422322e7a257/> updated generated test file
 - <csr-id-d807f943bc550df8a2cda4c246bbf765f1674065/> adjusted visibility
 - <csr-id-3580951b5faa8ef279291e5a6f994d1c9e0785d6/> cleaned up legacy naming
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings
 - <csr-id-180746ab27766c9df21b334482bead0da8f0bfba/> disabled mono-workflow on pull requests

### Chore

 - <csr-id-e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa/> generated changelogs

### New Features

 - <csr-id-bff97fe93ab537c2549893a33c8faa147dad0842/> added deep invocation, refactored runtime/engine names
 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-517b96da7ba93357229b7c1725ecb3331120c636/> decoupled telemetry from log output
 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-8fdef58ea207acb9ecb853c2c4934fe6daab39dd/> reorganized tracing span relationships
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-e5ed32378e0fd61c8bb1560027d252c0c93059a1/> added wick config dotviz, made interpreter tolerant of unused ports
 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65/> rounded out preliminary support for switch substreams
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-04d4fb0fc7137946fa10ee3e0f0be4c0cc73c8b3/> added ability to pass `with:` config to switch case operations
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-bd8af683437d46ed7281fd8cd806efe22ffa0f6f/> added quote-delimeted paths to field syntax, made rest router return errors on error packets
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-51d1da4a4ac6908fd1041ffd14ac7387b80b8ff6/> added arbitrary length plucked paths w/ support for array indices
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-85abe5adc703a9190b82dd78f58acdfe9920e3fe/> added better packet output in debugging mode
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-33c82afccdbcb4d7cda43e0ae880381501668478/> propagated seed to component context
 - <csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/> added context for wasm components
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-302612d5322fcc211b1ab7a05969c6de4bca7d7e/> added switch/case operation
 - <csr-id-0f05d770d08d86fc256154739b62ff089e26b503/> added sub-flow operatiions
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-daccbc3a2d42219d1004b0e6d9bbf134bd0a1142/> silencing warnings that do not give helpful info anyway
 - <csr-id-3208691ffb824e9f83d9845ae274c9b60bb8d4fa/> converted all level spans to info_spans
 - <csr-id-3b684528061d9c6a61ca4455415b96bfab0542dd/> fixed hanging tx for raw router components, removed bad debug line
 - <csr-id-d67c115ffe4ff7207c78cb03851c6f0e4793dfa4/> removed "slow tx" warning on healthy transactions
 - <csr-id-978690e2d05b2cae05991d273876a28f845abbb5/> fixed check in tests that resulted in duplicate DONE packets being sent
 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-7f2d0fbbb29191a235e3bcfeec2d19d62adab619/> fixed overflow on switch freeze, closes #376
 - <csr-id-c1c5e01b1ba3674cc328597b2cc5442bbd0d0b60/> replaced fixture
 - <csr-id-d6cc56690605804fb5660aeca7379477ebfab890/> fixed duplicate pluck ids causing Too many connections to input port 'input' errors
 - <csr-id-21863bff7f583df47a87dde689000f4d6dfc1a21/> fixed silenced errors from hushed modules
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-5f59bb11179ee19f49c82159e3b34f3abfe1c5ab/> fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-efdc1f0082b5cb73fa060d83e84d4bdb13f819a3/> fixed error on implicit db output:object, improved error details, renamed examples
 - <csr-id-34c2f4ebe5eee06d4fa999687a7327264bb957e7/> fixed source having an empty filename in error messages
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-c0ab15b0cf854a4ae8047c9f00d6da85febe0db2/> updated trace configuration, added jaeger endpoint to config.yaml settings
 - <csr-id-24dd80b7e1c6885687a7e16215055ee7e9b3fec6/> fixed switch implementation of held packets

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/> renamed transaction to executioncontext in interpreter
 - <csr-id-18a767a9d2e45c9efd1d3cbabe87b4450b78b255/> removed unnecessary circular reference
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle
 - <csr-id-39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f/> cleaned up intepreter, made some errors/warnings more clear
 - <csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/> updated rust-analyzer settings to be in line with CI checks, fixed lint errors
 - <csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/> removed conflicting timeouts in favor of per-op timeouts
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation
 - <csr-id-ff8b81dc1be6ff70237aaea1bc501b623f7c14d1/> merged PacketStream into Invocation for invocations

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 76 commits contributed to the release over the course of 130 calendar days.
 - 131 days passed between releases.
 - 74 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 7 unique issues were worked on: [#232](https://github.com/candlecorp/wick/issues/232), [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#341](https://github.com/candlecorp/wick/issues/341), [#347](https://github.com/candlecorp/wick/issues/347), [#396](https://github.com/candlecorp/wick/issues/396), [#400](https://github.com/candlecorp/wick/issues/400)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#341](https://github.com/candlecorp/wick/issues/341)**
    - Added ctx.inherent.timestamp, improved error message output ([`efe6055`](https://github.com/candlecorp/wick/commit/efe605510b846d2556f6060ba710fa154bdca7c4))
 * **[#347](https://github.com/candlecorp/wick/issues/347)**
    - Added `core::collect` component to collect a stream into a single object ([`2e6462a`](https://github.com/candlecorp/wick/commit/2e6462a8574ca5a09e0522ec7ff42ca4429657ba))
 * **[#396](https://github.com/candlecorp/wick/issues/396)**
    - Misc cli fixes ([`3c80d28`](https://github.com/candlecorp/wick/commit/3c80d28a266823034ad412580be4cec00ed80c36))
 * **[#400](https://github.com/candlecorp/wick/issues/400)**
    - Silencing warnings that do not give helpful info anyway ([`daccbc3`](https://github.com/candlecorp/wick/commit/daccbc3a2d42219d1004b0e6d9bbf134bd0a1142))
 * **Uncategorized**
    - Generated changelogs ([`e1d6c05`](https://github.com/candlecorp/wick/commit/e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa))
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Added deep invocation, refactored runtime/engine names ([`bff97fe`](https://github.com/candlecorp/wick/commit/bff97fe93ab537c2549893a33c8faa147dad0842))
    - Added wick audit & lockdown config ([`ddf1008`](https://github.com/candlecorp/wick/commit/ddf1008983c1f4a880a42ac4c29c0f60bc619cf3))
    - Decoupled telemetry from log output ([`517b96d`](https://github.com/candlecorp/wick/commit/517b96da7ba93357229b7c1725ecb3331120c636))
    - Support provides/requires relationship in composite components ([`8ceae1a`](https://github.com/candlecorp/wick/commit/8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d))
    - Converted all level spans to info_spans ([`3208691`](https://github.com/candlecorp/wick/commit/3208691ffb824e9f83d9845ae274c9b60bb8d4fa))
    - Renamed transaction to executioncontext in interpreter ([`37030ca`](https://github.com/candlecorp/wick/commit/37030caa9d8930774f6cac2f0b921d6f7d793941))
    - Reorganized tracing span relationships ([`8fdef58`](https://github.com/candlecorp/wick/commit/8fdef58ea207acb9ecb853c2c4934fe6daab39dd))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Fixed hanging tx for raw router components, removed bad debug line ([`3b68452`](https://github.com/candlecorp/wick/commit/3b684528061d9c6a61ca4455415b96bfab0542dd))
    - Removed "slow tx" warning on healthy transactions ([`d67c115`](https://github.com/candlecorp/wick/commit/d67c115ffe4ff7207c78cb03851c6f0e4793dfa4))
    - Fixed check in tests that resulted in duplicate DONE packets being sent ([`978690e`](https://github.com/candlecorp/wick/commit/978690e2d05b2cae05991d273876a28f845abbb5))
    - Added `wick config expand` ([`33ea9cd`](https://github.com/candlecorp/wick/commit/33ea9cd5fff9a85398e7fc15661cb9401a085c18))
    - Added wick config dotviz, made interpreter tolerant of unused ports ([`e5ed323`](https://github.com/candlecorp/wick/commit/e5ed32378e0fd61c8bb1560027d252c0c93059a1))
    - Added flow sequences, enhanced port inference ([`2a5cf0c`](https://github.com/candlecorp/wick/commit/2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695))
    - Added more tests for #378, fixed fields being requide by default from config ([`ae1400c`](https://github.com/candlecorp/wick/commit/ae1400caa092433bec0f66c04bd6e0efea30d173))
    - Fixed overflow on switch freeze, closes #376 ([`7f2d0fb`](https://github.com/candlecorp/wick/commit/7f2d0fbbb29191a235e3bcfeec2d19d62adab619))
    - Added inheritance/delegation to composite components, reorganized test files ([`e46db5f`](https://github.com/candlecorp/wick/commit/e46db5f2138254c227a2c39a3821074b77cf0166))
    - Removed unnecessary circular reference ([`18a767a`](https://github.com/candlecorp/wick/commit/18a767a9d2e45c9efd1d3cbabe87b4450b78b255))
    - Replaced fixture ([`c1c5e01`](https://github.com/candlecorp/wick/commit/c1c5e01b1ba3674cc328597b2cc5442bbd0d0b60))
    - Fixed duplicate pluck ids causing Too many connections to input port 'input' errors ([`d6cc566`](https://github.com/candlecorp/wick/commit/d6cc56690605804fb5660aeca7379477ebfab890))
    - Adjusted logging, interpreter execution lifecycle ([`316111a`](https://github.com/candlecorp/wick/commit/316111ac52d22365d060f573a456975de33b9115))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Updated generated test file ([`820e9ee`](https://github.com/candlecorp/wick/commit/820e9ee4d0b4f0126183f071d56a422322e7a257))
    - Rounded out preliminary support for switch substreams ([`a8232d0`](https://github.com/candlecorp/wick/commit/a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65))
    - Cleaned up intepreter, made some errors/warnings more clear ([`39f6a7d`](https://github.com/candlecorp/wick/commit/39f6a7d7d8a2079a5961eb2c550cd6e02d77e19f))
    - Adjusted visibility ([`d807f94`](https://github.com/candlecorp/wick/commit/d807f943bc550df8a2cda4c246bbf765f1674065))
    - Cleaned up legacy naming ([`3580951`](https://github.com/candlecorp/wick/commit/3580951b5faa8ef279291e5a6f994d1c9e0785d6))
    - Added ability to pass `with:` config to switch case operations ([`04d4fb0`](https://github.com/candlecorp/wick/commit/04d4fb0fc7137946fa10ee3e0f0be4c0cc73c8b3))
    - Fixed silenced errors from hushed modules ([`21863bf`](https://github.com/candlecorp/wick/commit/21863bff7f583df47a87dde689000f4d6dfc1a21))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - Updated rust-analyzer settings to be in line with CI checks, fixed lint errors ([`f5c8df4`](https://github.com/candlecorp/wick/commit/f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7))
    - Removed conflicting timeouts in favor of per-op timeouts ([`888814b`](https://github.com/candlecorp/wick/commit/888814bb24d3d4dd4b460af2616a72814f2bd7a1))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Added on_error & transaction support to ms sql server SQL implementation ([`d85d6f5`](https://github.com/candlecorp/wick/commit/d85d6f568d4548036c1af61e515c3fc187be6a6e))
    - Fixed quoted field syntax, empty JSON body decoding, increased wasm buffer size to 5mb ([`5f59bb1`](https://github.com/candlecorp/wick/commit/5f59bb11179ee19f49c82159e3b34f3abfe1c5ab))
    - Added quote-delimeted paths to field syntax, made rest router return errors on error packets ([`bd8af68`](https://github.com/candlecorp/wick/commit/bd8af683437d46ed7281fd8cd806efe22ffa0f6f))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Updated to rust 1.69.0, fixed associated warnings ([`e561fd6`](https://github.com/candlecorp/wick/commit/e561fd668afb1e1af3639c472a893b7fcfe2bf54))
    - Added arbitrary length plucked paths w/ support for array indices ([`51d1da4`](https://github.com/candlecorp/wick/commit/51d1da4a4ac6908fd1041ffd14ac7387b80b8ff6))
    - Added pluck shorthand where e.g. `op.name.input -> op.name` ([`262e0b5`](https://github.com/candlecorp/wick/commit/262e0b50c84229872ce7d1f006a878281b46d8e9))
    - Disabled mono-workflow on pull requests ([`180746a`](https://github.com/candlecorp/wick/commit/180746ab27766c9df21b334482bead0da8f0bfba))
    - Fixed error on implicit db output:object, improved error details, renamed examples ([`efdc1f0`](https://github.com/candlecorp/wick/commit/efdc1f0082b5cb73fa060d83e84d4bdb13f819a3))
    - Fixed source having an empty filename in error messages ([`34c2f4e`](https://github.com/candlecorp/wick/commit/34c2f4ebe5eee06d4fa999687a7327264bb957e7))
    - Added better packet output in debugging mode ([`85abe5a`](https://github.com/candlecorp/wick/commit/85abe5adc703a9190b82dd78f58acdfe9920e3fe))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Merged PacketStream into Invocation for invocations ([`ff8b81d`](https://github.com/candlecorp/wick/commit/ff8b81dc1be6ff70237aaea1bc501b623f7c14d1))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Updated trace configuration, added jaeger endpoint to config.yaml settings ([`c0ab15b`](https://github.com/candlecorp/wick/commit/c0ab15b0cf854a4ae8047c9f00d6da85febe0db2))
    - Added restapi router ([`58045d0`](https://github.com/candlecorp/wick/commit/58045d0fe75f519b84ebd45f3b1493e55fd4b282))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Fixed switch implementation of held packets ([`24dd80b`](https://github.com/candlecorp/wick/commit/24dd80b7e1c6885687a7e16215055ee7e9b3fec6))
    - Added proper type defs into config, closes #200. Fixed #228, #227 ([`49a53de`](https://github.com/candlecorp/wick/commit/49a53de6cb6631e2dc1f1e633d1c29d0510383cb))
    - Propagated seed to component context ([`33c82af`](https://github.com/candlecorp/wick/commit/33c82afccdbcb4d7cda43e0ae880381501668478))
    - Added context for wasm components ([`27c1fba`](https://github.com/candlecorp/wick/commit/27c1fba1d6af314e3b5f317178426331acc4b071))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added switch/case operation ([`302612d`](https://github.com/candlecorp/wick/commit/302612d5322fcc211b1ab7a05969c6de4bca7d7e))
    - Added sub-flow operatiions ([`0f05d77`](https://github.com/candlecorp/wick/commit/0f05d770d08d86fc256154739b62ff089e26b503))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
    - Added the ability to create inline node IDs in flow config ([`f7d7274`](https://github.com/candlecorp/wick/commit/f7d72741adae67477634ccdf52b93fe8f0c3c35f))
</details>

## v0.20.0 (2023-04-19)

<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-c724b06b8cf7776ba48b5a799d9e04e074d1c99d/>
<csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/>
<csr-id-406c10999648ca923fc8994b5835d11c823c19ce/>
<csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>
<csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/>
<csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/>

### Chore

 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-c724b06b8cf7776ba48b5a799d9e04e074d1c99d/> bumped deps, deprecated old crates, removed old kv component
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages
 - <csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/> renamed existing wafl references

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-0ce9f5573b827fa5e5d7d8dd5bac102e890a66e1/> propagated the network timeout to the interpreter
 - <csr-id-b83485305d609f9f599ae4a3f0aa03d9e101fb5c/> centralized relative file resolution within wick-config
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test
 - <csr-id-ade73755500573d2dec3ebf0e7113f73fa238549/> added pretty JSON output to wick invoke commands
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger
 - <csr-id-8745221bb0e25332f85bebe2387bc10a440ed5ac/> added codegen based off component.yaml
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Bug Fixes

 - <csr-id-1c58123f86ec95073b503790fe272b04003a05df/> adjusted default features on deps
 - <csr-id-16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc/> path resolution and missing wasm components in interpreter
 - <csr-id-5f346aade563554ddeb7b48c89c31dadc8ccfc5d/> fixed broken async tag for non-wasm targets
 - <csr-id-5c807f221fbb2eefaedaa899f82da3e8f2600388/> fixed broken tests

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable
 - <csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/> centralized APIs around configuration yaml

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup
 - <csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/> added registry tests, invoke tests, v1 tests

### Refactor (BREAKING)

 - <csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/> removed "default" value substitution in favor of a future impl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 30 commits contributed to the release over the course of 39 calendar days.
 - 24 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Adjusted default features on deps ([`1c58123`](https://github.com/candlecorp/wick/commit/1c58123f86ec95073b503790fe272b04003a05df))
    - Bumped deps, deprecated old crates, removed old kv component ([`c724b06`](https://github.com/candlecorp/wick/commit/c724b06b8cf7776ba48b5a799d9e04e074d1c99d))
    - Propagated the network timeout to the interpreter ([`0ce9f55`](https://github.com/candlecorp/wick/commit/0ce9f5573b827fa5e5d7d8dd5bac102e890a66e1))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Added registry tests, invoke tests, v1 tests ([`3802bf9`](https://github.com/candlecorp/wick/commit/3802bf93746725527d5dfa80f3c65d3314d4122c))
    - Path resolution and missing wasm components in interpreter ([`16bb6b4`](https://github.com/candlecorp/wick/commit/16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc))
    - Centralized relative file resolution within wick-config ([`b834853`](https://github.com/candlecorp/wick/commit/b83485305d609f9f599ae4a3f0aa03d9e101fb5c))
    - Centralized APIs around configuration yaml ([`fd71df4`](https://github.com/candlecorp/wick/commit/fd71df4baaa3f856454624396eff9d9ee8c4473f))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
    - Added pretty JSON output to wick invoke commands ([`ade7375`](https://github.com/candlecorp/wick/commit/ade73755500573d2dec3ebf0e7113f73fa238549))
    - Fixed broken async tag for non-wasm targets ([`5f346aa`](https://github.com/candlecorp/wick/commit/5f346aade563554ddeb7b48c89c31dadc8ccfc5d))
    - Fixed broken tests ([`5c807f2`](https://github.com/candlecorp/wick/commit/5c807f221fbb2eefaedaa899f82da3e8f2600388))
    - Removed "default" value substitution in favor of a future impl ([`c7b84da`](https://github.com/candlecorp/wick/commit/c7b84daacad21d9ba2c44123a6b0695db3b43528))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
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

