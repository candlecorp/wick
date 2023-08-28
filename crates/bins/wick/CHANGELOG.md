# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.14.0 (2023-08-28)

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-fd1b6d4cdc66e769d304d9168bb5575a6c5f930c/> consolidated deps to workspace
 - <csr-id-1b09917bf75ad3d954d4864bc3bf552137c3cd0f/> updated rustfmt and fixed formatting errors
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-599514816356f7fab3b2122156092166f7815427/> expanded tests to cover morme configuration cases
 - <csr-id-e561fd668afb1e1af3639c472a893b7fcfe2bf54/> updated to rust 1.69.0, fixed associated warnings
 - <csr-id-1c225e86ddc243751daf3b4c7d6404f16438919e/> renamed wick binary crate to wick-cli
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-7c4d1b26b7f07491c10d3c992b3142c899b15731/> increment to 0.6.0
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-c724b06b8cf7776ba48b5a799d9e04e074d1c99d/> bumped deps, deprecated old crates, removed old kv component
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages
 - <csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/> renamed existing wafl references

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-24152b7cc0002eac2ac1b0d75b545d5ca0b795b2/> made --with configs less strict so you can better leverage liquidjson to generate the config
 - <csr-id-bff97fe93ab537c2549893a33c8faa147dad0842/> added deep invocation, refactored runtime/engine names
 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-7ef0b24cf6112f3f11cd9309d545d38ab0ea9d28/> added better granularity to log filter rules
 - <csr-id-517b96da7ba93357229b7c1725ecb3331120c636/> decoupled telemetry from log output
 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-baee204ccdb6559798abcecee19362ea7b6bf80b/> made `wick run` the default subcommand if the first arg is invalid and exists as a file
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-e5ed32378e0fd61c8bb1560027d252c0c93059a1/> added wick config dotviz, made interpreter tolerant of unused ports
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-b679aad2e505e2e4b15794dc4decc98c51aee077/> added v1 wasm signatures, bumped wasmrs, enabled module cache
 - <csr-id-6a95f62937e51e2a82471f993cde8528add08121/> added memory profiler feature
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-1528f18c896c16ba798d37dcca5e017beecfd7c2/> added openapi spec generation
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-703988e288b32a1dc7f3d9dee232f4b4c79cc1cc/> made CLI parsing of arguments slightly smarter
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-4ddde01aea2936567f70f0dd16a0e23cd3d92b87/> change "--latest" to be "--tag=latest"
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-a4dfea5a6d76b3f8d6df83758ac8bff9f5e744e7/> made wick test output more intuitive, updated rust template
 - <csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/> added settings file, wick reg login, & wick reg push --latest
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/> added context for wasm components
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-947a6d9315cbfdcfd1e6780a47142b4273240b11/> wick run will run oci registry references
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/> adding tar.gz for extra files
 - <csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/> add Base64Bytes to wick-packet
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config
 - <csr-id-4c86477ce3176b546e06dc0e9db969921babe3d6/> added URL resource, migrated sql component to it
 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-b83485305d609f9f599ae4a3f0aa03d9e101fb5c/> centralized relative file resolution within wick-config
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test
 - <csr-id-ade73755500573d2dec3ebf0e7113f73fa238549/> added pretty JSON output to wick invoke commands
 - <csr-id-39fb923c30ec819bcbe665ef4fad569eebdfe194/> substreams/bracketing + codegen improvements
 - <csr-id-12cfaf9af0a36b9c42a59c922f0d447d832642ab/> added the ability to go from normalized config to serialized v1 config for init
 - <csr-id-9a7af1d91611e3625057fa4bf47982fa40b8191b/> added wick init, added rust component template
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger
 - <csr-id-8745221bb0e25332f85bebe2387bc10a440ed5ac/> added codegen based off component.yaml
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Bug Fixes

 - <csr-id-96ca1565d2966456f8b630fcca5333bb19085428/> made test revolve around a more static file
 - <csr-id-3239a4453868d04ea32ace557cc14ca75a3045e8/> reused existing imports in triggers and http routers
 - <csr-id-3208691ffb824e9f83d9845ae274c9b60bb8d4fa/> converted all level spans to info_spans
 - <csr-id-d901966927c3eec44270bbd2cd5d84baaa1f3462/> fixed relative volumes again
 - <csr-id-ce1eeaa918b9b49817cd1cf220dde0865c2ff97f/> fixed relative volume resources
 - <csr-id-516a395842bf80d81f17db30727ee2b5be69256f/> fixed ignored --filter argument on wick test
 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-efdc1f0082b5cb73fa060d83e84d4bdb13f819a3/> fixed error on implicit db output:object, improved error details, renamed examples
 - <csr-id-ee50fcf5a9ff0fafc8c19fba8c0f85be3afb51c3/> fixed settings-based auth on wick run and wick invoke
 - <csr-id-8603e25137b592c3e82ee80dd3ee5186f5fc8fb8/> clippy refactor
 - <csr-id-74b5a0474ffefaa1750ebc5c35d746ea4660f3d9/> same hash for latest registry push
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-f46d4a0ecb49b8e8b1802b48527b040224d2ff7a/> fixed precedence of CLI arg log level
 - <csr-id-c0ab15b0cf854a4ae8047c9f00d6da85febe0db2/> updated trace configuration, added jaeger endpoint to config.yaml settings
 - <csr-id-22d92b58500869729edda0283123800557057ed3/> fixed sql component with multiple inputs, incorrect signature match, fixes #238, #239
 - <csr-id-c1bb1d409adbf77c59da9e3241fa23d90cc39c8e/> added config fetch_all to fetch everything & made the default lazy
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues
 - <csr-id-66089ef51f87994a6a2be3a31f365f2226b81830/> changed postgres component to generic sql component
 - <csr-id-05d2aad6728b30a866d6fd30dfadc6626449ea38/> updated build job to defer to rust-toolchain.toml
 - <csr-id-16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc/> path resolution and missing wasm components in interpreter
 - <csr-id-5f346aade563554ddeb7b48c89c31dadc8ccfc5d/> fixed broken async tag for non-wasm targets

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------
 - <csr-id-3ce4d0fa1bb3decfbbc953b8dcab18cd5c6c5601/> release 0.7.0
 - <csr-id-141e17e389e92155ddc11e81f1bf374a2dd9d6f7/> fix cross-compilation fixes #190
 - <csr-id-3618a7ae8ac5006eb18fe543e791783120b9cae3/> change order for tag check to be first

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-cf597555a592d7d05b4541395d81e0eed5e35a10/> making global cache default for wick run
 - <csr-id-f76ecf1e1bc9ae4ec04c3df66b7fa15d0d2e3498/> consolidated include/exclude to one filter string
 - <csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/> renamed transaction to executioncontext in interpreter
 - <csr-id-a1b6b196a2b39ab10c126cf442115868a08b0fbf/> removed WASI CLI args
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-a988276c7fb02c4be6bd90a5762c76f788086364/> removed old pseudo-yield
 - <csr-id-316111ac52d22365d060f573a456975de33b9115/> adjusted logging, interpreter execution lifecycle
 - <csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/> removed conflicting timeouts in favor of per-op timeouts
 - <csr-id-12a0f6de257cf4b5789474fef448c7828f315bb5/> re-integrated Packet/PacketStream with Invocation
 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable
 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils
 - <csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/> centralized APIs around configuration yaml
 - <csr-id-11241c08c10e8fddb691a7130468c5974cda91f9/> reorganized wick-config structure to consolidate conversion code

### Test

 - <csr-id-5172449837c489f0231d4979ca4a5bb48f412aa2/> added cli test for wick test, fixed wasm test
 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup
 - <csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/> added registry tests, invoke tests, v1 tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 129 commits contributed to the release over the course of 170 calendar days.
 - 112 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 12 unique issues were worked on: [#144](https://github.com/candlecorp/wick/issues/144), [#154](https://github.com/candlecorp/wick/issues/154), [#171](https://github.com/candlecorp/wick/issues/171), [#194](https://github.com/candlecorp/wick/issues/194), [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#341](https://github.com/candlecorp/wick/issues/341), [#345](https://github.com/candlecorp/wick/issues/345), [#388](https://github.com/candlecorp/wick/issues/388), [#396](https://github.com/candlecorp/wick/issues/396), [#399](https://github.com/candlecorp/wick/issues/399), [#405](https://github.com/candlecorp/wick/issues/405)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **[#154](https://github.com/candlecorp/wick/issues/154)**
    - Nightly releases ([`1f074ba`](https://github.com/candlecorp/wick/commit/1f074babbd4045b579f63a0d3dc67d8675093247))
 * **[#171](https://github.com/candlecorp/wick/issues/171)**
    - Change order for tag check to be first ([`3618a7a`](https://github.com/candlecorp/wick/commit/3618a7ae8ac5006eb18fe543e791783120b9cae3))
 * **[#194](https://github.com/candlecorp/wick/issues/194)**
    - Fix cross-compilation fixes #190 ([`141e17e`](https://github.com/candlecorp/wick/commit/141e17e389e92155ddc11e81f1bf374a2dd9d6f7))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#341](https://github.com/candlecorp/wick/issues/341)**
    - Added ctx.inherent.timestamp, improved error message output ([`efe6055`](https://github.com/candlecorp/wick/commit/efe605510b846d2556f6060ba710fa154bdca7c4))
 * **[#345](https://github.com/candlecorp/wick/issues/345)**
    - Added `exec`-style SQL operation ([`1162c1d`](https://github.com/candlecorp/wick/commit/1162c1d4bef87d585d76be7bb4b55811aa946796))
 * **[#388](https://github.com/candlecorp/wick/issues/388)**
    - Added `wick install` ([`3158048`](https://github.com/candlecorp/wick/commit/3158048ad1d0c33518cb647d08f927606afcecd0))
 * **[#396](https://github.com/candlecorp/wick/issues/396)**
    - Misc cli fixes ([`3c80d28`](https://github.com/candlecorp/wick/commit/3c80d28a266823034ad412580be4cec00ed80c36))
 * **[#399](https://github.com/candlecorp/wick/issues/399)**
    - Better http client substream support. ([`744f1ac`](https://github.com/candlecorp/wick/commit/744f1ac3d5fa8c28e8e0a1e80d7f5e49839c0c43))
 * **[#405](https://github.com/candlecorp/wick/issues/405)**
    - Fixed "refusing to overwrite ..." errors on application runs. ([`a10242d`](https://github.com/candlecorp/wick/commit/a10242d4786cfa199eaf61289b9da99d09c114a7))
 * **Uncategorized**
    - 0.14.0 release ([`159876e`](https://github.com/candlecorp/wick/commit/159876e237175595e4eb317aa8d4c702b006f5c0))
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Made test revolve around a more static file ([`96ca156`](https://github.com/candlecorp/wick/commit/96ca1565d2966456f8b630fcca5333bb19085428))
    - Made --with configs less strict so you can better leverage liquidjson to generate the config ([`24152b7`](https://github.com/candlecorp/wick/commit/24152b7cc0002eac2ac1b0d75b545d5ca0b795b2))
    - Added deep invocation, refactored runtime/engine names ([`bff97fe`](https://github.com/candlecorp/wick/commit/bff97fe93ab537c2549893a33c8faa147dad0842))
    - Increment to v13 ([`ce31f6e`](https://github.com/candlecorp/wick/commit/ce31f6eed2fad0d91a8c59f490d6961671e5c38d))
    - Making global cache default for wick run ([`cf59755`](https://github.com/candlecorp/wick/commit/cf597555a592d7d05b4541395d81e0eed5e35a10))
    - Added wick audit & lockdown config ([`ddf1008`](https://github.com/candlecorp/wick/commit/ddf1008983c1f4a880a42ac4c29c0f60bc619cf3))
    - Reused existing imports in triggers and http routers ([`3239a44`](https://github.com/candlecorp/wick/commit/3239a4453868d04ea32ace557cc14ca75a3045e8))
    - Added better granularity to log filter rules ([`7ef0b24`](https://github.com/candlecorp/wick/commit/7ef0b24cf6112f3f11cd9309d545d38ab0ea9d28))
    - Consolidated include/exclude to one filter string ([`f76ecf1`](https://github.com/candlecorp/wick/commit/f76ecf1e1bc9ae4ec04c3df66b7fa15d0d2e3498))
    - Decoupled telemetry from log output ([`517b96d`](https://github.com/candlecorp/wick/commit/517b96da7ba93357229b7c1725ecb3331120c636))
    - Support provides/requires relationship in composite components ([`8ceae1a`](https://github.com/candlecorp/wick/commit/8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d))
    - Converted all level spans to info_spans ([`3208691`](https://github.com/candlecorp/wick/commit/3208691ffb824e9f83d9845ae274c9b60bb8d4fa))
    - Renamed transaction to executioncontext in interpreter ([`37030ca`](https://github.com/candlecorp/wick/commit/37030caa9d8930774f6cac2f0b921d6f7d793941))
    - Update Cargo.toml ([`7e41ff7`](https://github.com/candlecorp/wick/commit/7e41ff70d67a090c01d16c99e7e317b29eb7724f))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Consolidated deps to workspace ([`fd1b6d4`](https://github.com/candlecorp/wick/commit/fd1b6d4cdc66e769d304d9168bb5575a6c5f930c))
    - Made `wick run` the default subcommand if the first arg is invalid and exists as a file ([`baee204`](https://github.com/candlecorp/wick/commit/baee204ccdb6559798abcecee19362ea7b6bf80b))
    - Fixed relative volumes again ([`d901966`](https://github.com/candlecorp/wick/commit/d901966927c3eec44270bbd2cd5d84baaa1f3462))
    - Fixed relative volume resources ([`ce1eeaa`](https://github.com/candlecorp/wick/commit/ce1eeaa918b9b49817cd1cf220dde0865c2ff97f))
    - Removed WASI CLI args ([`a1b6b19`](https://github.com/candlecorp/wick/commit/a1b6b196a2b39ab10c126cf442115868a08b0fbf))
    - Added `wick config expand` ([`33ea9cd`](https://github.com/candlecorp/wick/commit/33ea9cd5fff9a85398e7fc15661cb9401a085c18))
    - Added wick config dotviz, made interpreter tolerant of unused ports ([`e5ed323`](https://github.com/candlecorp/wick/commit/e5ed32378e0fd61c8bb1560027d252c0c93059a1))
    - Added inheritance/delegation to composite components, reorganized test files ([`e46db5f`](https://github.com/candlecorp/wick/commit/e46db5f2138254c227a2c39a3821074b77cf0166))
    - Eliminated fetching of bytes before checking cache ([`586ace0`](https://github.com/candlecorp/wick/commit/586ace0978ca8adf58bf4d1fa5ed392015297c21))
    - Removed old pseudo-yield ([`a988276`](https://github.com/candlecorp/wick/commit/a988276c7fb02c4be6bd90a5762c76f788086364))
    - Added v1 wasm signatures, bumped wasmrs, enabled module cache ([`b679aad`](https://github.com/candlecorp/wick/commit/b679aad2e505e2e4b15794dc4decc98c51aee077))
    - Added memory profiler feature ([`6a95f62`](https://github.com/candlecorp/wick/commit/6a95f62937e51e2a82471f993cde8528add08121))
    - Adjusted logging, interpreter execution lifecycle ([`316111a`](https://github.com/candlecorp/wick/commit/316111ac52d22365d060f573a456975de33b9115))
    - Updated rustfmt and fixed formatting errors ([`1b09917`](https://github.com/candlecorp/wick/commit/1b09917bf75ad3d954d4864bc3bf552137c3cd0f))
    - Fixed ignored --filter argument on wick test ([`516a395`](https://github.com/candlecorp/wick/commit/516a395842bf80d81f17db30727ee2b5be69256f))
    - Fixed included cached assets on wick reg push ([`4577461`](https://github.com/candlecorp/wick/commit/4577461e0a767ec99ae6482c2e2efeb3069ca0c8))
    - Added openapi spec generation ([`1528f18`](https://github.com/candlecorp/wick/commit/1528f18c896c16ba798d37dcca5e017beecfd7c2))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - V0.10.0 ([`26980ba`](https://github.com/candlecorp/wick/commit/26980ba04606d62fbfe0bacf529d80b696750c6c))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Removed conflicting timeouts in favor of per-op timeouts ([`888814b`](https://github.com/candlecorp/wick/commit/888814bb24d3d4dd4b460af2616a72814f2bd7a1))
    - Added on_error & transaction support to ms sql server SQL implementation ([`d85d6f5`](https://github.com/candlecorp/wick/commit/d85d6f568d4548036c1af61e515c3fc187be6a6e))
    - Update Cargo.toml ([`5918ca8`](https://github.com/candlecorp/wick/commit/5918ca83712f2d85a4d5d0437d52cf0444fd4e2a))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Made CLI parsing of arguments slightly smarter ([`703988e`](https://github.com/candlecorp/wick/commit/703988e288b32a1dc7f3d9dee232f4b4c79cc1cc))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Change "--latest" to be "--tag=latest" ([`4ddde01`](https://github.com/candlecorp/wick/commit/4ddde01aea2936567f70f0dd16a0e23cd3d92b87))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Expanded tests to cover morme configuration cases ([`5995148`](https://github.com/candlecorp/wick/commit/599514816356f7fab3b2122156092166f7815427))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Updated to rust 1.69.0, fixed associated warnings ([`e561fd6`](https://github.com/candlecorp/wick/commit/e561fd668afb1e1af3639c472a893b7fcfe2bf54))
    - Made wick test output more intuitive, updated rust template ([`a4dfea5`](https://github.com/candlecorp/wick/commit/a4dfea5a6d76b3f8d6df83758ac8bff9f5e744e7))
    - Fixed error on implicit db output:object, improved error details, renamed examples ([`efdc1f0`](https://github.com/candlecorp/wick/commit/efdc1f0082b5cb73fa060d83e84d4bdb13f819a3))
    - Fixed settings-based auth on wick run and wick invoke ([`ee50fcf`](https://github.com/candlecorp/wick/commit/ee50fcf5a9ff0fafc8c19fba8c0f85be3afb51c3))
    - Clippy refactor ([`8603e25`](https://github.com/candlecorp/wick/commit/8603e25137b592c3e82ee80dd3ee5186f5fc8fb8))
    - Same hash for latest registry push ([`74b5a04`](https://github.com/candlecorp/wick/commit/74b5a0474ffefaa1750ebc5c35d746ea4660f3d9))
    - Re-integrated Packet/PacketStream with Invocation ([`12a0f6d`](https://github.com/candlecorp/wick/commit/12a0f6de257cf4b5789474fef448c7828f315bb5))
    - Update Cargo.toml ([`b73714b`](https://github.com/candlecorp/wick/commit/b73714b6a44c7771ee0ba5f725cdf2f4143ea27a))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Fixed precedence of CLI arg log level ([`f46d4a0`](https://github.com/candlecorp/wick/commit/f46d4a0ecb49b8e8b1802b48527b040224d2ff7a))
    - Updated trace configuration, added jaeger endpoint to config.yaml settings ([`c0ab15b`](https://github.com/candlecorp/wick/commit/c0ab15b0cf854a4ae8047c9f00d6da85febe0db2))
    - Added settings file, wick reg login, & wick reg push --latest ([`63858e1`](https://github.com/candlecorp/wick/commit/63858e1bc6673b61d50fa8f66dc4378369850910))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Fixed sql component with multiple inputs, incorrect signature match, fixes #238, #239 ([`22d92b5`](https://github.com/candlecorp/wick/commit/22d92b58500869729edda0283123800557057ed3))
    - Added proper type defs into config, closes #200. Fixed #228, #227 ([`49a53de`](https://github.com/candlecorp/wick/commit/49a53de6cb6631e2dc1f1e633d1c29d0510383cb))
    - Release 0.7.0 ([`3ce4d0f`](https://github.com/candlecorp/wick/commit/3ce4d0fa1bb3decfbbc953b8dcab18cd5c6c5601))
    - Added context for wasm components ([`27c1fba`](https://github.com/candlecorp/wick/commit/27c1fba1d6af314e3b5f317178426331acc4b071))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added config fetch_all to fetch everything & made the default lazy ([`c1bb1d4`](https://github.com/candlecorp/wick/commit/c1bb1d409adbf77c59da9e3241fa23d90cc39c8e))
    - Added glob support in package files ([`53ff1dd`](https://github.com/candlecorp/wick/commit/53ff1dd49057a0b7cb45deff02b350d8f1b2970e))
    - Fixed linting issues ([`6c6f9a8`](https://github.com/candlecorp/wick/commit/6c6f9a80f9873f5989453c7800a355724cb61fff))
    - Wick run will run oci registry references ([`947a6d9`](https://github.com/candlecorp/wick/commit/947a6d9315cbfdcfd1e6780a47142b4273240b11))
    - Added gzip error handling ([`6fb111c`](https://github.com/candlecorp/wick/commit/6fb111cc0068ca5a4709ef274b046c0b590eee08))
    - Adding tar.gz for extra files ([`8c58c35`](https://github.com/candlecorp/wick/commit/8c58c354e765a51abb602b184c45055b9d561ed5))
    - Add Base64Bytes to wick-packet ([`399c5d5`](https://github.com/candlecorp/wick/commit/399c5d518b0a291dba63fb3f69337af2911d1776))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
    - Added type imports ([`17c9058`](https://github.com/candlecorp/wick/commit/17c9058b98935fa8ed29dbc27b899c9e3244eb67))
    - Added cli test for wick test, fixed wasm test ([`5172449`](https://github.com/candlecorp/wick/commit/5172449837c489f0231d4979ca4a5bb48f412aa2))
    - Added the ability to create inline node IDs in flow config ([`f7d7274`](https://github.com/candlecorp/wick/commit/f7d72741adae67477634ccdf52b93fe8f0c3c35f))
    - Added URL resource, migrated sql component to it ([`4c86477`](https://github.com/candlecorp/wick/commit/4c86477ce3176b546e06dc0e9db969921babe3d6))
    - Renamed wick binary crate to wick-cli ([`1c225e8`](https://github.com/candlecorp/wick/commit/1c225e86ddc243751daf3b4c7d6404f16438919e))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Increment to 0.6.0 ([`7c4d1b2`](https://github.com/candlecorp/wick/commit/7c4d1b26b7f07491c10d3c992b3142c899b15731))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Changed postgres component to generic sql component ([`66089ef`](https://github.com/candlecorp/wick/commit/66089ef51f87994a6a2be3a31f365f2226b81830))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Updated build job to defer to rust-toolchain.toml ([`05d2aad`](https://github.com/candlecorp/wick/commit/05d2aad6728b30a866d6fd30dfadc6626449ea38))
    - Bumped deps, deprecated old crates, removed old kv component ([`c724b06`](https://github.com/candlecorp/wick/commit/c724b06b8cf7776ba48b5a799d9e04e074d1c99d))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Added registry tests, invoke tests, v1 tests ([`3802bf9`](https://github.com/candlecorp/wick/commit/3802bf93746725527d5dfa80f3c65d3314d4122c))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
    - Path resolution and missing wasm components in interpreter ([`16bb6b4`](https://github.com/candlecorp/wick/commit/16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc))
    - Centralized relative file resolution within wick-config ([`b834853`](https://github.com/candlecorp/wick/commit/b83485305d609f9f599ae4a3f0aa03d9e101fb5c))
    - Centralized APIs around configuration yaml ([`fd71df4`](https://github.com/candlecorp/wick/commit/fd71df4baaa3f856454624396eff9d9ee8c4473f))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
    - Added pretty JSON output to wick invoke commands ([`ade7375`](https://github.com/candlecorp/wick/commit/ade73755500573d2dec3ebf0e7113f73fa238549))
    - Fixed broken async tag for non-wasm targets ([`5f346aa`](https://github.com/candlecorp/wick/commit/5f346aade563554ddeb7b48c89c31dadc8ccfc5d))
    - Substreams/bracketing + codegen improvements ([`39fb923`](https://github.com/candlecorp/wick/commit/39fb923c30ec819bcbe665ef4fad569eebdfe194))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Added the ability to go from normalized config to serialized v1 config for init ([`12cfaf9`](https://github.com/candlecorp/wick/commit/12cfaf9af0a36b9c42a59c922f0d447d832642ab))
    - Reorganized wick-config structure to consolidate conversion code ([`11241c0`](https://github.com/candlecorp/wick/commit/11241c08c10e8fddb691a7130468c5974cda91f9))
    - Added wick init, added rust component template ([`9a7af1d`](https://github.com/candlecorp/wick/commit/9a7af1d91611e3625057fa4bf47982fa40b8191b))
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

