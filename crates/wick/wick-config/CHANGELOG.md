# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.28.0 (2023-10-18)

### Chore

 - <csr-id-35ff51b8a93c27475765a7eb65c23256f4f93d67/> updated versions and changelogs
 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-6cbc8b53e1f68fa5336220261fc80f0256601133/> added experimental settings section, removed incomplete example
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/> fixed warnings

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog
 - <csr-id-e343e7d9bfc02d3ee817f596f4fdf184db087046/> update docs for cloud
 - <csr-id-f1360f859e13dc49f6e6978f606e1315f1cf370e/> updated generated markdown for enums
 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs
 - <csr-id-10672c5db34d10e50869b2c14977f9235761cabd/> updated config codegen, refactored config for clarity, fixed template

### New Features

 - <csr-id-11449d002b80fbc22ec5e4b684b09fbcc949a9c7/> added support for wasm component-model triggers
 - <csr-id-ee711616cfaa412433b975bcf14791bcb198d712/> added first-pass at TypeScript config SDK
 - <csr-id-dc38b405ef148e8ed6d991b567b497e2d07368ea/> added http client proxy support
 - <csr-id-7ca53308add4e920c0e8ce3755ec62c56ceedb80/> added optional directory listing for the static server
 - <csr-id-2ce019fed2c7d9348c9c47d5221d322e700ce293/> added support for wasm imports
 - <csr-id-7bacdb9a4559e3de86e0a17544e76634ffe4de28/> made generating v1 configs wasm-compatible
 - <csr-id-8760659095ce1f0f9a0bbd835bcf34827b21317c/> added __dirname, consolidated loose render events
 - <csr-id-4516bb7034d4dbe0ffbe6625df32302d40e63570/> support volume restrictions on file:// urls, in-mem SQLite DBs
 - <csr-id-72a2fb3af224ff0b674c8e75a8c6e94070c181a7/> added packet assertions to wick test cases
 - <csr-id-24152b7cc0002eac2ac1b0d75b545d5ca0b795b2/> made --with configs less strict so you can better leverage liquidjson to generate the config
 - <csr-id-bff97fe93ab537c2549893a33c8faa147dad0842/> added deep invocation, refactored runtime/engine names
 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-70f0fd07ac70ae4fd1bb1734b306266f14f3af3c/> made buffer_size configurable
 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-8fdef58ea207acb9ecb853c2c4934fe6daab39dd/> reorganized tracing span relationships
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-dd57e5062f3cf5d01e163ad104e56f7debc50aa4/> added xml codec for wick-http-component
 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-cc404a0dd2006e63fbd399c8c8ae5d12cec55913/> made name in test definitions optional
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-1528f18c896c16ba798d37dcca5e017beecfd7c2/> added openapi spec generation
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-64e30fbb7e64e7f744190ebcbab107b4916a24e1/> better discriminated HTTP errors, removed error output from 500 responses
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-e2abceed2d1cc7436fbe4631d3eac861ae91675e/> updated headers to be liquidjson
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-f4f04af492c7e0fe90472a6a5bafebfdbeddf622/> added types to package
 - <csr-id-103c9d8e67fff895d02c10597faedfe8b72d1eab/> added fallback option for static http
   * feat: added fallback option for static http
   
   * fix: fix clippy error
   
   * refactor: cleaned up code for style
   
   * fix: corrected documentation
   
   * fix: remove async from response function
 - <csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/> added settings file, wick reg login, & wick reg push --latest
 - <csr-id-ba2015ddf2d24324c311fa681a39c4a65ac886bc/> added azure-sql support
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-e08b20481d197c3ceff74b7d42eabecef1ef3c78/> added rest router config
 - <csr-id-5495686f598e766a73c240554e5c8fbdfb297376/> added form-data codec to http client
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-ba94e4dd43a85bb0dd79953f92b5a053e1536e62/> added op config to http client operations, added builders for config types
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/> added context for wasm components
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/> added asset flags, fixed relative volumes, fixed manifest locations
 - <csr-id-302612d5322fcc211b1ab7a05969c6de4bca7d7e/> added switch/case operation
 - <csr-id-0f05d770d08d86fc256154739b62ff089e26b503/> added sub-flow operatiions
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/> adding tar.gz for extra files
 - <csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/> add Base64Bytes to wick-packet
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports
 - <csr-id-cbd6515303db5bb5fb9383116f0ee69a90e4c537/> added reverse proxy router
 - <csr-id-16940c8908ef9a463c227d8e8fdd5c1ad6bfc379/> added static router
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config
 - <csr-id-4c86477ce3176b546e06dc0e9db969921babe3d6/> added URL resource, migrated sql component to it

### Bug Fixes

 - <csr-id-9b6380ebb0a5f82e8c06784890f05e1f80908804/> flattened $defs in JSON schema generation
 - <csr-id-7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6/> fixed config rendering within trigger operations
 - <csr-id-3239a4453868d04ea32ace557cc14ca75a3045e8/> reused existing imports in triggers and http routers
 - <csr-id-d901966927c3eec44270bbd2cd5d84baaa1f3462/> fixed relative volumes again
 - <csr-id-ce1eeaa918b9b49817cd1cf220dde0865c2ff97f/> fixed relative volume resources
 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-3108cf583cf49a93b706be93ce87c47f77633727/> corrected openapi path + replaced name with id in rest router config
 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-e107d7cc2fb3d36925fe8af471b164c07ec3e15d/> fixed broken cache path, fixed unrendered Volume configuraton
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-ce2837aaacbd70d43c7f87150790f72880ac0703/> reordered error behavior variants to make ignore default
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-44d10001d8d3464963dd7e1872d49d98113950d3/> added `registry` as alias for `host` in package and `data` as alias for `value` in tests
 - <csr-id-34c2f4ebe5eee06d4fa999687a7327264bb957e7/> fixed source having an empty filename in error messages
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-76331ad61955d86a5776b742f7cec8d163daeb2f/> derived asset traits on resource bindings
 - <csr-id-8a481a3f749ac4102f5041aefff94b1363893cd4/> pass oci credentials to fetch
 - <csr-id-28c625552c460ac5c337efad3b0d621c9ec593cc/> don't fetch lazy assets
 - <csr-id-fdf152db1a352c48b75d08b4d4187a748c7f0795/> removed dependency on jq
 - <csr-id-c1bb1d409adbf77c59da9e3241fa23d90cc39c8e/> added config fetch_all to fetch everything & made the default lazy
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Refactor

 - <csr-id-2a29e9bf1c22e751df6981cfc208b2336d2dd65b/> removed old, dead code
 - <csr-id-69d79c1c8eee66dcd766648c359145a1898691c7/> removed native stdlib and associated references
 - <csr-id-42a39c2b9150b56e27c8b7b41cccebc0cef09015/> pulled triggers into their own crates
 - <csr-id-644c2ffde3be9b39bd087147d2e6599fbb6c1c85/> made generic Binding struct
 - <csr-id-43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5/> lowercased the start character of all log events
 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-be57f85e388c38265c33d457339c4dbf5f1ae65f/> renamed XML codec to Text
 - <csr-id-6aecefa7d7fe4e806b239cf9cadb914837c10dbe/> removed experimental block, changed expose to extends
 - <csr-id-33527b199a5057e0bf9d51c6e5a4068b9a8dc830/> improved reliability and tolerance of wick test execution
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/> updated rust-analyzer settings to be in line with CI checks, fixed lint errors
 - <csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/> removed conflicting timeouts in favor of per-op timeouts
 - <csr-id-b590d66e24d1e1dd582656b54b896586e9c8f4fb/> adjusted data types, fixed code-genned files

### Test

 - <csr-id-5172449837c489f0231d4979ca4a5bb48f412aa2/> added cli test for wick test, fixed wasm test

### New Features (BREAKING)

 - <csr-id-534d209c797d962d4fd90d590ecdb5916ecede56/> made ComponentError anyhow::Error

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 122 commits contributed to the release over the course of 180 calendar days.
 - 182 days passed between releases.
 - 116 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 9 unique issues were worked on: [#221](https://github.com/candlecorp/wick/issues/221), [#232](https://github.com/candlecorp/wick/issues/232), [#254](https://github.com/candlecorp/wick/issues/254), [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#341](https://github.com/candlecorp/wick/issues/341), [#345](https://github.com/candlecorp/wick/issues/345), [#375](https://github.com/candlecorp/wick/issues/375), [#405](https://github.com/candlecorp/wick/issues/405)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#221](https://github.com/candlecorp/wick/issues/221)**
    - Added vscode extension ([`3bdd785`](https://github.com/candlecorp/wick/commit/3bdd7855a16535e809c6e868be2f2d3c45cacb13))
 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#254](https://github.com/candlecorp/wick/issues/254)**
    - Added fallback option for static http ([`103c9d8`](https://github.com/candlecorp/wick/commit/103c9d8e67fff895d02c10597faedfe8b72d1eab))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#341](https://github.com/candlecorp/wick/issues/341)**
    - Added ctx.inherent.timestamp, improved error message output ([`efe6055`](https://github.com/candlecorp/wick/commit/efe605510b846d2556f6060ba710fa154bdca7c4))
 * **[#345](https://github.com/candlecorp/wick/issues/345)**
    - Added `exec`-style SQL operation ([`1162c1d`](https://github.com/candlecorp/wick/commit/1162c1d4bef87d585d76be7bb4b55811aa946796))
 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **[#405](https://github.com/candlecorp/wick/issues/405)**
    - Fixed "refusing to overwrite ..." errors on application runs. ([`a10242d`](https://github.com/candlecorp/wick/commit/a10242d4786cfa199eaf61289b9da99d09c114a7))
 * **Uncategorized**
    - Removed old, dead code ([`2a29e9b`](https://github.com/candlecorp/wick/commit/2a29e9bf1c22e751df6981cfc208b2336d2dd65b))
    - Removed native stdlib and associated references ([`69d79c1`](https://github.com/candlecorp/wick/commit/69d79c1c8eee66dcd766648c359145a1898691c7))
    - Added support for wasm component-model triggers ([`11449d0`](https://github.com/candlecorp/wick/commit/11449d002b80fbc22ec5e4b684b09fbcc949a9c7))
    - Pulled triggers into their own crates ([`42a39c2`](https://github.com/candlecorp/wick/commit/42a39c2b9150b56e27c8b7b41cccebc0cef09015))
    - Added first-pass at TypeScript config SDK ([`ee71161`](https://github.com/candlecorp/wick/commit/ee711616cfaa412433b975bcf14791bcb198d712))
    - Added http client proxy support ([`dc38b40`](https://github.com/candlecorp/wick/commit/dc38b405ef148e8ed6d991b567b497e2d07368ea))
    - Updated versions and changelogs ([`35ff51b`](https://github.com/candlecorp/wick/commit/35ff51b8a93c27475765a7eb65c23256f4f93d67))
    - Added optional directory listing for the static server ([`7ca5330`](https://github.com/candlecorp/wick/commit/7ca53308add4e920c0e8ce3755ec62c56ceedb80))
    - Flattened $defs in JSON schema generation ([`9b6380e`](https://github.com/candlecorp/wick/commit/9b6380ebb0a5f82e8c06784890f05e1f80908804))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`4d6e3f4`](https://github.com/candlecorp/wick/commit/4d6e3f437964552cfd6917310c17548b12e83eaf))
    - Made ComponentError anyhow::Error ([`534d209`](https://github.com/candlecorp/wick/commit/534d209c797d962d4fd90d590ecdb5916ecede56))
    - Added support for wasm imports ([`2ce019f`](https://github.com/candlecorp/wick/commit/2ce019fed2c7d9348c9c47d5221d322e700ce293))
    - Made generic Binding struct ([`644c2ff`](https://github.com/candlecorp/wick/commit/644c2ffde3be9b39bd087147d2e6599fbb6c1c85))
    - Made generating v1 configs wasm-compatible ([`7bacdb9`](https://github.com/candlecorp/wick/commit/7bacdb9a4559e3de86e0a17544e76634ffe4de28))
    - Migrated AsRef<str> to concrete types or Into<String> ([`60128f7`](https://github.com/candlecorp/wick/commit/60128f7707f2d2a537ffa32e24376f58d7faa7be))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Added __dirname, consolidated loose render events ([`8760659`](https://github.com/candlecorp/wick/commit/8760659095ce1f0f9a0bbd835bcf34827b21317c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Fixed config rendering within trigger operations ([`7d0a399`](https://github.com/candlecorp/wick/commit/7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6))
    - Lowercased the start character of all log events ([`43fa508`](https://github.com/candlecorp/wick/commit/43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Support volume restrictions on file:// urls, in-mem SQLite DBs ([`4516bb7`](https://github.com/candlecorp/wick/commit/4516bb7034d4dbe0ffbe6625df32302d40e63570))
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Added packet assertions to wick test cases ([`72a2fb3`](https://github.com/candlecorp/wick/commit/72a2fb3af224ff0b674c8e75a8c6e94070c181a7))
    - Made --with configs less strict so you can better leverage liquidjson to generate the config ([`24152b7`](https://github.com/candlecorp/wick/commit/24152b7cc0002eac2ac1b0d75b545d5ca0b795b2))
    - Added deep invocation, refactored runtime/engine names ([`bff97fe`](https://github.com/candlecorp/wick/commit/bff97fe93ab537c2549893a33c8faa147dad0842))
    - Update docs for cloud ([`e343e7d`](https://github.com/candlecorp/wick/commit/e343e7d9bfc02d3ee817f596f4fdf184db087046))
    - Added wick audit & lockdown config ([`ddf1008`](https://github.com/candlecorp/wick/commit/ddf1008983c1f4a880a42ac4c29c0f60bc619cf3))
    - Reused existing imports in triggers and http routers ([`3239a44`](https://github.com/candlecorp/wick/commit/3239a4453868d04ea32ace557cc14ca75a3045e8))
    - Made buffer_size configurable ([`70f0fd0`](https://github.com/candlecorp/wick/commit/70f0fd07ac70ae4fd1bb1734b306266f14f3af3c))
    - Support provides/requires relationship in composite components ([`8ceae1a`](https://github.com/candlecorp/wick/commit/8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d))
    - Reorganized tracing span relationships ([`8fdef58`](https://github.com/candlecorp/wick/commit/8fdef58ea207acb9ecb853c2c4934fe6daab39dd))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Fixed relative volumes again ([`d901966`](https://github.com/candlecorp/wick/commit/d901966927c3eec44270bbd2cd5d84baaa1f3462))
    - Renamed XML codec to Text ([`be57f85`](https://github.com/candlecorp/wick/commit/be57f85e388c38265c33d457339c4dbf5f1ae65f))
    - Fixed relative volume resources ([`ce1eeaa`](https://github.com/candlecorp/wick/commit/ce1eeaa918b9b49817cd1cf220dde0865c2ff97f))
    - Added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax ([`b0b9cd2`](https://github.com/candlecorp/wick/commit/b0b9cd20f748ffe1956ad2501fe23991fededf13))
    - Re-added exposing volumes to WASI components ([`ce9d202`](https://github.com/candlecorp/wick/commit/ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85))
    - Added `wick config expand` ([`33ea9cd`](https://github.com/candlecorp/wick/commit/33ea9cd5fff9a85398e7fc15661cb9401a085c18))
    - Added flow sequences, enhanced port inference ([`2a5cf0c`](https://github.com/candlecorp/wick/commit/2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695))
    - Added more tests for #378, fixed fields being requide by default from config ([`ae1400c`](https://github.com/candlecorp/wick/commit/ae1400caa092433bec0f66c04bd6e0efea30d173))
    - Removed experimental block, changed expose to extends ([`6aecefa`](https://github.com/candlecorp/wick/commit/6aecefa7d7fe4e806b239cf9cadb914837c10dbe))
    - Added experimental settings section, removed incomplete example ([`6cbc8b5`](https://github.com/candlecorp/wick/commit/6cbc8b53e1f68fa5336220261fc80f0256601133))
    - Added inheritance/delegation to composite components, reorganized test files ([`e46db5f`](https://github.com/candlecorp/wick/commit/e46db5f2138254c227a2c39a3821074b77cf0166))
    - Added xml codec for wick-http-component ([`dd57e50`](https://github.com/candlecorp/wick/commit/dd57e5062f3cf5d01e163ad104e56f7debc50aa4))
    - Improved reliability and tolerance of wick test execution ([`33527b1`](https://github.com/candlecorp/wick/commit/33527b199a5057e0bf9d51c6e5a4068b9a8dc830))
    - Corrected openapi path + replaced name with id in rest router config ([`3108cf5`](https://github.com/candlecorp/wick/commit/3108cf583cf49a93b706be93ce87c47f77633727))
    - Eliminated fetching of bytes before checking cache ([`586ace0`](https://github.com/candlecorp/wick/commit/586ace0978ca8adf58bf4d1fa5ed392015297c21))
    - Added unions to type definitions ([`222cc7f`](https://github.com/candlecorp/wick/commit/222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f))
    - Made name in test definitions optional ([`cc404a0`](https://github.com/candlecorp/wick/commit/cc404a0dd2006e63fbd399c8c8ae5d12cec55913))
    - Fixed included cached assets on wick reg push ([`4577461`](https://github.com/candlecorp/wick/commit/4577461e0a767ec99ae6482c2e2efeb3069ca0c8))
    - Fixed broken cache path, fixed unrendered Volume configuraton ([`e107d7c`](https://github.com/candlecorp/wick/commit/e107d7cc2fb3d36925fe8af471b164c07ec3e15d))
    - Added openapi spec generation ([`1528f18`](https://github.com/candlecorp/wick/commit/1528f18c896c16ba798d37dcca5e017beecfd7c2))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - Reordered error behavior variants to make ignore default ([`ce2837a`](https://github.com/candlecorp/wick/commit/ce2837aaacbd70d43c7f87150790f72880ac0703))
    - Updated generated markdown for enums ([`f1360f8`](https://github.com/candlecorp/wick/commit/f1360f859e13dc49f6e6978f606e1315f1cf370e))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Updated rust-analyzer settings to be in line with CI checks, fixed lint errors ([`f5c8df4`](https://github.com/candlecorp/wick/commit/f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7))
    - Removed conflicting timeouts in favor of per-op timeouts ([`888814b`](https://github.com/candlecorp/wick/commit/888814bb24d3d4dd4b460af2616a72814f2bd7a1))
    - Added configurable timeout per-operation ([`d0d58be`](https://github.com/candlecorp/wick/commit/d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d))
    - Added on_error & transaction support to ms sql server SQL implementation ([`d85d6f5`](https://github.com/candlecorp/wick/commit/d85d6f568d4548036c1af61e515c3fc187be6a6e))
    - Better discriminated HTTP errors, removed error output from 500 responses ([`64e30fb`](https://github.com/candlecorp/wick/commit/64e30fbb7e64e7f744190ebcbab107b4916a24e1))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Added `registry` as alias for `host` in package and `data` as alias for `value` in tests ([`44d1000`](https://github.com/candlecorp/wick/commit/44d10001d8d3464963dd7e1872d49d98113950d3))
    - Updated config codegen, refactored config for clarity, fixed template ([`10672c5`](https://github.com/candlecorp/wick/commit/10672c5db34d10e50869b2c14977f9235761cabd))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Propagating component config through to user code ([`8058284`](https://github.com/candlecorp/wick/commit/8058284a1a686366fa8829f9377981d7ba389554))
    - Adjusted data types, fixed code-genned files ([`b590d66`](https://github.com/candlecorp/wick/commit/b590d66e24d1e1dd582656b54b896586e9c8f4fb))
    - Updated headers to be liquidjson ([`e2abcee`](https://github.com/candlecorp/wick/commit/e2abceed2d1cc7436fbe4631d3eac861ae91675e))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Added pluck shorthand where e.g. `op.name.input -> op.name` ([`262e0b5`](https://github.com/candlecorp/wick/commit/262e0b50c84229872ce7d1f006a878281b46d8e9))
    - Fixed warnings ([`ab7d535`](https://github.com/candlecorp/wick/commit/ab7d5355945adb592c4e00ccdc8b268e146e6535))
    - Added types to package ([`f4f04af`](https://github.com/candlecorp/wick/commit/f4f04af492c7e0fe90472a6a5bafebfdbeddf622))
    - Fixed source having an empty filename in error messages ([`34c2f4e`](https://github.com/candlecorp/wick/commit/34c2f4ebe5eee06d4fa999687a7327264bb957e7))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Added settings file, wick reg login, & wick reg push --latest ([`63858e1`](https://github.com/candlecorp/wick/commit/63858e1bc6673b61d50fa8f66dc4378369850910))
    - Added azure-sql support ([`ba2015d`](https://github.com/candlecorp/wick/commit/ba2015ddf2d24324c311fa681a39c4a65ac886bc))
    - Added restapi router ([`58045d0`](https://github.com/candlecorp/wick/commit/58045d0fe75f519b84ebd45f3b1493e55fd4b282))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Added rest router config ([`e08b204`](https://github.com/candlecorp/wick/commit/e08b20481d197c3ceff74b7d42eabecef1ef3c78))
    - Added form-data codec to http client ([`5495686`](https://github.com/candlecorp/wick/commit/5495686f598e766a73c240554e5c8fbdfb297376))
    - Added op config to http client operations, added builders for config types ([`ba94e4d`](https://github.com/candlecorp/wick/commit/ba94e4dd43a85bb0dd79953f92b5a053e1536e62))
    - Added proper type defs into config, closes #200. Fixed #228, #227 ([`49a53de`](https://github.com/candlecorp/wick/commit/49a53de6cb6631e2dc1f1e633d1c29d0510383cb))
    - Derived asset traits on resource bindings ([`76331ad`](https://github.com/candlecorp/wick/commit/76331ad61955d86a5776b742f7cec8d163daeb2f))
    - Pass oci credentials to fetch ([`8a481a3`](https://github.com/candlecorp/wick/commit/8a481a3f749ac4102f5041aefff94b1363893cd4))
    - Don't fetch lazy assets ([`28c6255`](https://github.com/candlecorp/wick/commit/28c625552c460ac5c337efad3b0d621c9ec593cc))
    - Added context for wasm components ([`27c1fba`](https://github.com/candlecorp/wick/commit/27c1fba1d6af314e3b5f317178426331acc4b071))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Removed dependency on jq ([`fdf152d`](https://github.com/candlecorp/wick/commit/fdf152db1a352c48b75d08b4d4187a748c7f0795))
    - Added asset flags, fixed relative volumes, fixed manifest locations ([`3dd4cdb`](https://github.com/candlecorp/wick/commit/3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08))
    - Added switch/case operation ([`302612d`](https://github.com/candlecorp/wick/commit/302612d5322fcc211b1ab7a05969c6de4bca7d7e))
    - Added sub-flow operatiions ([`0f05d77`](https://github.com/candlecorp/wick/commit/0f05d770d08d86fc256154739b62ff089e26b503))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Added config fetch_all to fetch everything & made the default lazy ([`c1bb1d4`](https://github.com/candlecorp/wick/commit/c1bb1d409adbf77c59da9e3241fa23d90cc39c8e))
    - Added glob support in package files ([`53ff1dd`](https://github.com/candlecorp/wick/commit/53ff1dd49057a0b7cb45deff02b350d8f1b2970e))
    - Fixed linting issues ([`6c6f9a8`](https://github.com/candlecorp/wick/commit/6c6f9a80f9873f5989453c7800a355724cb61fff))
    - Added gzip error handling ([`6fb111c`](https://github.com/candlecorp/wick/commit/6fb111cc0068ca5a4709ef274b046c0b590eee08))
    - Adding tar.gz for extra files ([`8c58c35`](https://github.com/candlecorp/wick/commit/8c58c354e765a51abb602b184c45055b9d561ed5))
    - Add Base64Bytes to wick-packet ([`399c5d5`](https://github.com/candlecorp/wick/commit/399c5d518b0a291dba63fb3f69337af2911d1776))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
    - Added type imports ([`17c9058`](https://github.com/candlecorp/wick/commit/17c9058b98935fa8ed29dbc27b899c9e3244eb67))
    - Added cli test for wick test, fixed wasm test ([`5172449`](https://github.com/candlecorp/wick/commit/5172449837c489f0231d4979ca4a5bb48f412aa2))
    - Added reverse proxy router ([`cbd6515`](https://github.com/candlecorp/wick/commit/cbd6515303db5bb5fb9383116f0ee69a90e4c537))
    - Added static router ([`16940c8`](https://github.com/candlecorp/wick/commit/16940c8908ef9a463c227d8e8fdd5c1ad6bfc379))
    - Added the ability to create inline node IDs in flow config ([`f7d7274`](https://github.com/candlecorp/wick/commit/f7d72741adae67477634ccdf52b93fe8f0c3c35f))
    - Added URL resource, migrated sql component to it ([`4c86477`](https://github.com/candlecorp/wick/commit/4c86477ce3176b546e06dc0e9db969921babe3d6))
</details>

## v0.27.0 (2023-09-14)

<csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/>
<csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/>
<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-6cbc8b53e1f68fa5336220261fc80f0256601133/>
<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/>
<csr-id-644c2ffde3be9b39bd087147d2e6599fbb6c1c85/>
<csr-id-43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-be57f85e388c38265c33d457339c4dbf5f1ae65f/>
<csr-id-6aecefa7d7fe4e806b239cf9cadb914837c10dbe/>
<csr-id-33527b199a5057e0bf9d51c6e5a4068b9a8dc830/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/>
<csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/>
<csr-id-b590d66e24d1e1dd582656b54b896586e9c8f4fb/>
<csr-id-5172449837c489f0231d4979ca4a5bb48f412aa2/>

### Chore

 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-6cbc8b53e1f68fa5336220261fc80f0256601133/> added experimental settings section, removed incomplete example
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/> fixed warnings

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog
 - <csr-id-e343e7d9bfc02d3ee817f596f4fdf184db087046/> update docs for cloud
 - <csr-id-f1360f859e13dc49f6e6978f606e1315f1cf370e/> updated generated markdown for enums
 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs
 - <csr-id-10672c5db34d10e50869b2c14977f9235761cabd/> updated config codegen, refactored config for clarity, fixed template

### New Features

<csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/>
<csr-id-ba2015ddf2d24324c311fa681a39c4a65ac886bc/>
<csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/>
<csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/>
<csr-id-e08b20481d197c3ceff74b7d42eabecef1ef3c78/>
<csr-id-5495686f598e766a73c240554e5c8fbdfb297376/>
<csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/>
<csr-id-ba94e4dd43a85bb0dd79953f92b5a053e1536e62/>
<csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/>
<csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/>
<csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/>
<csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/>
<csr-id-302612d5322fcc211b1ab7a05969c6de4bca7d7e/>
<csr-id-0f05d770d08d86fc256154739b62ff089e26b503/>
<csr-id-027392a9514ba4846e068b21476e980ea53bee1d/>
<csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/>
<csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/>
<csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/>
<csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/>
<csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/>
<csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/>
<csr-id-cbd6515303db5bb5fb9383116f0ee69a90e4c537/>
<csr-id-16940c8908ef9a463c227d8e8fdd5c1ad6bfc379/>
<csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/>
<csr-id-4c86477ce3176b546e06dc0e9db969921babe3d6/>

 - <csr-id-7ca53308add4e920c0e8ce3755ec62c56ceedb80/> added optional directory listing for the static server
 - <csr-id-2ce019fed2c7d9348c9c47d5221d322e700ce293/> added support for wasm imports
 - <csr-id-7bacdb9a4559e3de86e0a17544e76634ffe4de28/> made generating v1 configs wasm-compatible
 - <csr-id-8760659095ce1f0f9a0bbd835bcf34827b21317c/> added __dirname, consolidated loose render events
 - <csr-id-4516bb7034d4dbe0ffbe6625df32302d40e63570/> support volume restrictions on file:// urls, in-mem SQLite DBs
 - <csr-id-72a2fb3af224ff0b674c8e75a8c6e94070c181a7/> added packet assertions to wick test cases
 - <csr-id-24152b7cc0002eac2ac1b0d75b545d5ca0b795b2/> made --with configs less strict so you can better leverage liquidjson to generate the config
 - <csr-id-bff97fe93ab537c2549893a33c8faa147dad0842/> added deep invocation, refactored runtime/engine names
 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-70f0fd07ac70ae4fd1bb1734b306266f14f3af3c/> made buffer_size configurable
 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-8fdef58ea207acb9ecb853c2c4934fe6daab39dd/> reorganized tracing span relationships
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-dd57e5062f3cf5d01e163ad104e56f7debc50aa4/> added xml codec for wick-http-component
 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-cc404a0dd2006e63fbd399c8c8ae5d12cec55913/> made name in test definitions optional
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-1528f18c896c16ba798d37dcca5e017beecfd7c2/> added openapi spec generation
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-64e30fbb7e64e7f744190ebcbab107b4916a24e1/> better discriminated HTTP errors, removed error output from 500 responses
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-e2abceed2d1cc7436fbe4631d3eac861ae91675e/> updated headers to be liquidjson
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-f4f04af492c7e0fe90472a6a5bafebfdbeddf622/> added types to package
 - <csr-id-103c9d8e67fff895d02c10597faedfe8b72d1eab/> added fallback option for static http
   * feat: added fallback option for static http
* fix: fix clippy error
* refactor: cleaned up code for style
* fix: corrected documentation
* fix: remove async from response function

### Bug Fixes

 - <csr-id-9b6380ebb0a5f82e8c06784890f05e1f80908804/> flattened $defs in JSON schema generation
 - <csr-id-7d0a399741cc1f0ab1b876cc6a31ad00fc1a58c6/> fixed config rendering within trigger operations
 - <csr-id-3239a4453868d04ea32ace557cc14ca75a3045e8/> reused existing imports in triggers and http routers
 - <csr-id-d901966927c3eec44270bbd2cd5d84baaa1f3462/> fixed relative volumes again
 - <csr-id-ce1eeaa918b9b49817cd1cf220dde0865c2ff97f/> fixed relative volume resources
 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-3108cf583cf49a93b706be93ce87c47f77633727/> corrected openapi path + replaced name with id in rest router config
 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-e107d7cc2fb3d36925fe8af471b164c07ec3e15d/> fixed broken cache path, fixed unrendered Volume configuraton
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-ce2837aaacbd70d43c7f87150790f72880ac0703/> reordered error behavior variants to make ignore default
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-44d10001d8d3464963dd7e1872d49d98113950d3/> added `registry` as alias for `host` in package and `data` as alias for `value` in tests
 - <csr-id-34c2f4ebe5eee06d4fa999687a7327264bb957e7/> fixed source having an empty filename in error messages
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-76331ad61955d86a5776b742f7cec8d163daeb2f/> derived asset traits on resource bindings
 - <csr-id-8a481a3f749ac4102f5041aefff94b1363893cd4/> pass oci credentials to fetch
 - <csr-id-28c625552c460ac5c337efad3b0d621c9ec593cc/> don't fetch lazy assets
 - <csr-id-fdf152db1a352c48b75d08b4d4187a748c7f0795/> removed dependency on jq
 - <csr-id-c1bb1d409adbf77c59da9e3241fa23d90cc39c8e/> added config fetch_all to fetch everything & made the default lazy
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Refactor

 - <csr-id-644c2ffde3be9b39bd087147d2e6599fbb6c1c85/> made generic Binding struct
 - <csr-id-43fa5081c09f1e4003f550c6ae62bfcc50d6e6f5/> lowercased the start character of all log events
 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-be57f85e388c38265c33d457339c4dbf5f1ae65f/> renamed XML codec to Text
 - <csr-id-6aecefa7d7fe4e806b239cf9cadb914837c10dbe/> removed experimental block, changed expose to extends
 - <csr-id-33527b199a5057e0bf9d51c6e5a4068b9a8dc830/> improved reliability and tolerance of wick test execution
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/> updated rust-analyzer settings to be in line with CI checks, fixed lint errors
 - <csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/> removed conflicting timeouts in favor of per-op timeouts
 - <csr-id-b590d66e24d1e1dd582656b54b896586e9c8f4fb/> adjusted data types, fixed code-genned files

### Test

 - <csr-id-5172449837c489f0231d4979ca4a5bb48f412aa2/> added cli test for wick test, fixed wasm test

### New Features (BREAKING)

 - <csr-id-534d209c797d962d4fd90d590ecdb5916ecede56/> made ComponentError anyhow::Error

<csr-unknown>
 added settings file, wick reg login, & wick reg push –latest added azure-sql support added restapi router normalized accessor api for wick-config added rest router config added form-data codec to http client added codec to HTTP server, added runtime constraints, ability to explicitly drop packets added op config to http client operations, added builders for config types added proper type defs into config, closes #200. Fixed #228, #227 added context for wasm components added operation context added asset flags, fixed relative volumes, fixed manifest locations added switch/case operation added sub-flow operatiions added pluck & merge added glob support in package files added gzip error handling adding tar.gz for extra files add Base64Bytes to wick-packet added http client component added type imports added reverse proxy router added static router added the ability to create inline node IDs in flow config added URL resource, migrated sql component to it<csr-unknown/>

## v0.26.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-6cbc8b53e1f68fa5336220261fc80f0256601133/>
<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>
<csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-be57f85e388c38265c33d457339c4dbf5f1ae65f/>
<csr-id-6aecefa7d7fe4e806b239cf9cadb914837c10dbe/>
<csr-id-33527b199a5057e0bf9d51c6e5a4068b9a8dc830/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>
<csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/>
<csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/>
<csr-id-b590d66e24d1e1dd582656b54b896586e9c8f4fb/>
<csr-id-5172449837c489f0231d4979ca4a5bb48f412aa2/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-6cbc8b53e1f68fa5336220261fc80f0256601133/> added experimental settings section, removed incomplete example
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named
 - <csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/> fixed warnings

### Documentation

 - <csr-id-e343e7d9bfc02d3ee817f596f4fdf184db087046/> update docs for cloud
 - <csr-id-f1360f859e13dc49f6e6978f606e1315f1cf370e/> updated generated markdown for enums
 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs
 - <csr-id-10672c5db34d10e50869b2c14977f9235761cabd/> updated config codegen, refactored config for clarity, fixed template

### New Features

<csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/>
<csr-id-ba2015ddf2d24324c311fa681a39c4a65ac886bc/>
<csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/>
<csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/>
<csr-id-e08b20481d197c3ceff74b7d42eabecef1ef3c78/>
<csr-id-5495686f598e766a73c240554e5c8fbdfb297376/>
<csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/>
<csr-id-ba94e4dd43a85bb0dd79953f92b5a053e1536e62/>
<csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/>
<csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/>
<csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/>
<csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/>
<csr-id-302612d5322fcc211b1ab7a05969c6de4bca7d7e/>
<csr-id-0f05d770d08d86fc256154739b62ff089e26b503/>
<csr-id-027392a9514ba4846e068b21476e980ea53bee1d/>
<csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/>
<csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/>
<csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/>
<csr-id-399c5d518b0a291dba63fb3f69337af2911d1776/>
<csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/>
<csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/>
<csr-id-cbd6515303db5bb5fb9383116f0ee69a90e4c537/>
<csr-id-16940c8908ef9a463c227d8e8fdd5c1ad6bfc379/>
<csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/>
<csr-id-4c86477ce3176b546e06dc0e9db969921babe3d6/>

 - <csr-id-4516bb7034d4dbe0ffbe6625df32302d40e63570/> support volume restrictions on file:// urls, in-mem SQLite DBs
 - <csr-id-72a2fb3af224ff0b674c8e75a8c6e94070c181a7/> added packet assertions to wick test cases
 - <csr-id-24152b7cc0002eac2ac1b0d75b545d5ca0b795b2/> made --with configs less strict so you can better leverage liquidjson to generate the config
 - <csr-id-bff97fe93ab537c2549893a33c8faa147dad0842/> added deep invocation, refactored runtime/engine names
 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-70f0fd07ac70ae4fd1bb1734b306266f14f3af3c/> made buffer_size configurable
 - <csr-id-8ceae1a2a357b34d10eafe9295d7b4b6ae8d4b4d/> support provides/requires relationship in composite components
 - <csr-id-8fdef58ea207acb9ecb853c2c4934fe6daab39dd/> reorganized tracing span relationships
 - <csr-id-b0b9cd20f748ffe1956ad2501fe23991fededf13/> added sqlite support, added inline ids for queries, normalized ms sql $1->@p1 syntax
 - <csr-id-ce9d2020b4a1a8397ae2013b05f8de4fd1e96a85/> re-added exposing volumes to WASI components
 - <csr-id-33ea9cd5fff9a85398e7fc15661cb9401a085c18/> added `wick config expand`
 - <csr-id-2a5cf0c1adcd6aacd083967da9e8e7c6c46a9695/> added flow sequences, enhanced port inference
 - <csr-id-e46db5f2138254c227a2c39a3821074b77cf0166/> added inheritance/delegation to composite components, reorganized test files
 - <csr-id-dd57e5062f3cf5d01e163ad104e56f7debc50aa4/> added xml codec for wick-http-component
 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-cc404a0dd2006e63fbd399c8c8ae5d12cec55913/> made name in test definitions optional
 - <csr-id-efe605510b846d2556f6060ba710fa154bdca7c4/> added ctx.inherent.timestamp, improved error message output
 - <csr-id-1528f18c896c16ba798d37dcca5e017beecfd7c2/> added openapi spec generation
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d0d58bed91a911c19a8fcd54d2ec5f9a6fd1d74d/> added configurable timeout per-operation
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-64e30fbb7e64e7f744190ebcbab107b4916a24e1/> better discriminated HTTP errors, removed error output from 500 responses
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-8058284a1a686366fa8829f9377981d7ba389554/> propagating component config through to user code
 - <csr-id-e2abceed2d1cc7436fbe4631d3eac861ae91675e/> updated headers to be liquidjson
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-262e0b50c84229872ce7d1f006a878281b46d8e9/> added pluck shorthand where e.g. `op.name.input -> op.name`
 - <csr-id-f4f04af492c7e0fe90472a6a5bafebfdbeddf622/> added types to package
 - <csr-id-103c9d8e67fff895d02c10597faedfe8b72d1eab/> added fallback option for static http
   * feat: added fallback option for static http
* fix: fix clippy error
* refactor: cleaned up code for style
* fix: corrected documentation
* fix: remove async from response function

### Bug Fixes

 - <csr-id-3239a4453868d04ea32ace557cc14ca75a3045e8/> reused existing imports in triggers and http routers
 - <csr-id-d901966927c3eec44270bbd2cd5d84baaa1f3462/> fixed relative volumes again
 - <csr-id-ce1eeaa918b9b49817cd1cf220dde0865c2ff97f/> fixed relative volume resources
 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-3108cf583cf49a93b706be93ce87c47f77633727/> corrected openapi path + replaced name with id in rest router config
 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-e107d7cc2fb3d36925fe8af471b164c07ec3e15d/> fixed broken cache path, fixed unrendered Volume configuraton
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-ce2837aaacbd70d43c7f87150790f72880ac0703/> reordered error behavior variants to make ignore default
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-44d10001d8d3464963dd7e1872d49d98113950d3/> added `registry` as alias for `host` in package and `data` as alias for `value` in tests
 - <csr-id-34c2f4ebe5eee06d4fa999687a7327264bb957e7/> fixed source having an empty filename in error messages
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-76331ad61955d86a5776b742f7cec8d163daeb2f/> derived asset traits on resource bindings
 - <csr-id-8a481a3f749ac4102f5041aefff94b1363893cd4/> pass oci credentials to fetch
 - <csr-id-28c625552c460ac5c337efad3b0d621c9ec593cc/> don't fetch lazy assets
 - <csr-id-fdf152db1a352c48b75d08b4d4187a748c7f0795/> removed dependency on jq
 - <csr-id-c1bb1d409adbf77c59da9e3241fa23d90cc39c8e/> added config fetch_all to fetch everything & made the default lazy
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-be57f85e388c38265c33d457339c4dbf5f1ae65f/> renamed XML codec to Text
 - <csr-id-6aecefa7d7fe4e806b239cf9cadb914837c10dbe/> removed experimental block, changed expose to extends
 - <csr-id-33527b199a5057e0bf9d51c6e5a4068b9a8dc830/> improved reliability and tolerance of wick test execution
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache
 - <csr-id-f5c8df4f1ec673b8e8811c8d03e0ad68e85fabd7/> updated rust-analyzer settings to be in line with CI checks, fixed lint errors
 - <csr-id-888814bb24d3d4dd4b460af2616a72814f2bd7a1/> removed conflicting timeouts in favor of per-op timeouts
 - <csr-id-b590d66e24d1e1dd582656b54b896586e9c8f4fb/> adjusted data types, fixed code-genned files

### Test

 - <csr-id-5172449837c489f0231d4979ca4a5bb48f412aa2/> added cli test for wick test, fixed wasm test

<csr-unknown>
 added settings file, wick reg login, & wick reg push –latest added azure-sql support added restapi router normalized accessor api for wick-config added rest router config added form-data codec to http client added codec to HTTP server, added runtime constraints, ability to explicitly drop packets added op config to http client operations, added builders for config types added proper type defs into config, closes #200. Fixed #228, #227 added context for wasm components added operation context added asset flags, fixed relative volumes, fixed manifest locations added switch/case operation added sub-flow operatiions added pluck & merge added glob support in package files added gzip error handling adding tar.gz for extra files add Base64Bytes to wick-packet added http client component added type imports added reverse proxy router added static router added the ability to create inline node IDs in flow config added URL resource, migrated sql component to it<csr-unknown/>
<csr-unknown/>

## v0.26.0 (2023-04-18)

<csr-id-b123bf44c34987b15dcabb1e9ef2f882c68cf5ae/>
<csr-id-7361b149ca108904341364426e1509105913f31f/>
<csr-id-a39072526072d0649e598078a4449e4b856a49d1/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-7e2538202a03999c2b5781d7658b72118dce9446/>
<csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/>
<csr-id-88fd22885f2357d409d6d3d1d3b244ee65b69a67/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>
<csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/>

### Chore

 - <csr-id-b123bf44c34987b15dcabb1e9ef2f882c68cf5ae/> removing wick-package as a dev-dependency of wick-config
 - <csr-id-7361b149ca108904341364426e1509105913f31f/> release
   flow-component, flow-expression-parser, flow-graph, wick-asset-reference, wick-component, wick-config, wick-oci-utils
 - <csr-id-a39072526072d0649e598078a4449e4b856a49d1/> fixed udeps logic, removed unused dep from wick-config
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-154c09b8b1169cb92bbc35135ab516e42c51e5d0/> add bdd and time trigger
 - <csr-id-10335669483d0498968cdabe194e11d6c4907c19/> integrated config and asset creates
 - <csr-id-b83485305d609f9f599ae4a3f0aa03d9e101fb5c/> centralized relative file resolution within wick-config
 - <csr-id-a73e506365455fa8fd707e00ea659ecaa96cffb7/> added discrimant to determine configuration type
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test

### Bug Fixes

 - <csr-id-66089ef51f87994a6a2be3a31f365f2226b81830/> changed postgres component to generic sql component
 - <csr-id-46c3bd67a13e349280d16ce50c336a5415ef589c/> clippy style
 - <csr-id-4947e4665e1decc34f226ad0c116b8259c7c2153/> improved error handling of wick-package
 - <csr-id-16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc/> path resolution and missing wasm components in interpreter

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable
 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils
 - <csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/> centralized APIs around configuration yaml
 - <csr-id-88fd22885f2357d409d6d3d1d3b244ee65b69a67/> cleaned up visibility and exports for wick-config

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup
 - <csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/> added registry tests, invoke tests, v1 tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 21 commits contributed to the release over the course of 19 calendar days.
 - 26 days passed between releases.
 - 21 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Removing wick-package as a dev-dependency of wick-config ([`b123bf4`](https://github.com/candlecorp/wick/commit/b123bf44c34987b15dcabb1e9ef2f882c68cf5ae))
    - Release ([`7361b14`](https://github.com/candlecorp/wick/commit/7361b149ca108904341364426e1509105913f31f))
    - Fixed udeps logic, removed unused dep from wick-config ([`a390725`](https://github.com/candlecorp/wick/commit/a39072526072d0649e598078a4449e4b856a49d1))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Changed postgres component to generic sql component ([`66089ef`](https://github.com/candlecorp/wick/commit/66089ef51f87994a6a2be3a31f365f2226b81830))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Clippy style ([`46c3bd6`](https://github.com/candlecorp/wick/commit/46c3bd67a13e349280d16ce50c336a5415ef589c))
    - Add bdd and time trigger ([`154c09b`](https://github.com/candlecorp/wick/commit/154c09b8b1169cb92bbc35135ab516e42c51e5d0))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Added registry tests, invoke tests, v1 tests ([`3802bf9`](https://github.com/candlecorp/wick/commit/3802bf93746725527d5dfa80f3c65d3314d4122c))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
    - Improved error handling of wick-package ([`4947e46`](https://github.com/candlecorp/wick/commit/4947e4665e1decc34f226ad0c116b8259c7c2153))
    - Integrated config and asset creates ([`1033566`](https://github.com/candlecorp/wick/commit/10335669483d0498968cdabe194e11d6c4907c19))
    - Path resolution and missing wasm components in interpreter ([`16bb6b4`](https://github.com/candlecorp/wick/commit/16bb6b4e60436ab7a0ee931e89e3e9485fbe32dc))
    - Centralized relative file resolution within wick-config ([`b834853`](https://github.com/candlecorp/wick/commit/b83485305d609f9f599ae4a3f0aa03d9e101fb5c))
    - Centralized APIs around configuration yaml ([`fd71df4`](https://github.com/candlecorp/wick/commit/fd71df4baaa3f856454624396eff9d9ee8c4473f))
    - Cleaned up visibility and exports for wick-config ([`88fd228`](https://github.com/candlecorp/wick/commit/88fd22885f2357d409d6d3d1d3b244ee65b69a67))
    - Added discrimant to determine configuration type ([`a73e506`](https://github.com/candlecorp/wick/commit/a73e506365455fa8fd707e00ea659ecaa96cffb7))
    - Added wick-test ([`bc79d37`](https://github.com/candlecorp/wick/commit/bc79d37c98b41e10815a9641396e73b3c4c3b55a))
</details>

## v0.24.0 (2023-03-23)

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

## v0.23.0 (2023-03-23)

<csr-id-f229d8ee9dbb1c051d18b911bb4ef868b968ea14/>
<csr-id-406c10999648ca923fc8994b5835d11c823c19ce/>
<csr-id-88c97a7ddca56ace4e7aeacbc2dcc4d47a0b11d0/>
<csr-id-11241c08c10e8fddb691a7130468c5974cda91f9/>
<csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/>

### Chore

 - <csr-id-f229d8ee9dbb1c051d18b911bb4ef868b968ea14/> Release
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages
 - <csr-id-88c97a7ddca56ace4e7aeacbc2dcc4d47a0b11d0/> removed dead code

### New Features

 - <csr-id-12cfaf9af0a36b9c42a59c922f0d447d832642ab/> added the ability to go from normalized config to serialized v1 config for init
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger

### Refactor

 - <csr-id-11241c08c10e8fddb691a7130468c5974cda91f9/> reorganized wick-config structure to consolidate conversion code

### Refactor (BREAKING)

 - <csr-id-c7b84daacad21d9ba2c44123a6b0695db3b43528/> removed "default" value substitution in favor of a future impl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 2 calendar days.
 - 8 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`f229d8e`](https://github.com/candlecorp/wick/commit/f229d8ee9dbb1c051d18b911bb4ef868b968ea14))
    - Removed "default" value substitution in favor of a future impl ([`c7b84da`](https://github.com/candlecorp/wick/commit/c7b84daacad21d9ba2c44123a6b0695db3b43528))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Added the ability to go from normalized config to serialized v1 config for init ([`12cfaf9`](https://github.com/candlecorp/wick/commit/12cfaf9af0a36b9c42a59c922f0d447d832642ab))
    - Reorganized wick-config structure to consolidate conversion code ([`11241c0`](https://github.com/candlecorp/wick/commit/11241c08c10e8fddb691a7130468c5974cda91f9))
    - Removed dead code ([`88c97a7`](https://github.com/candlecorp/wick/commit/88c97a7ddca56ace4e7aeacbc2dcc4d47a0b11d0))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
</details>

## v0.22.0 (2023-03-15)

<csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/>
<csr-id-c27131154861be5625b82e1e7d99d8228df1fa39/>

### Chore

 - <csr-id-3a42e6388e3561103412ca3e47db8b5feb5ef3a9/> renamed existing wafl references

### New Features

 - <csr-id-8745221bb0e25332f85bebe2387bc10a440ed5ac/> added codegen based off component.yaml
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Other

 - <csr-id-c27131154861be5625b82e1e7d99d8228df1fa39/> added http types example as tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 4 calendar days.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Renamed existing wafl references ([`3a42e63`](https://github.com/candlecorp/wick/commit/3a42e6388e3561103412ca3e47db8b5feb5ef3a9))
    - Added codegen based off component.yaml ([`8745221`](https://github.com/candlecorp/wick/commit/8745221bb0e25332f85bebe2387bc10a440ed5ac))
    - Added http types example as tests ([`c271311`](https://github.com/candlecorp/wick/commit/c27131154861be5625b82e1e7d99d8228df1fa39))
    - Incorporated interface.json into component.yaml ([`97280ee`](https://github.com/candlecorp/wick/commit/97280ee71b361472dbb6ae32c77626b07c218554))
    - Shoring up tests. fixed error propagation and hung txs stemming from timeouts ([`46310b9`](https://github.com/candlecorp/wick/commit/46310b98b6933c5a6d84c32863391bb482af5ac3))
    - Renamed wick-config-component to wick-config, added app config, restructured triggers, added trigger test component ([`24ef43f`](https://github.com/candlecorp/wick/commit/24ef43f7fc978c1f33f27a1e90f9971abdeb9b11))
</details>

