# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.17.0 (2023-10-18)

### Chore

 - <csr-id-35ff51b8a93c27475765a7eb65c23256f4f93d67/> updated versions and changelogs
 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog
 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-8b8c990bc76df17fad61c8fe903e64a3c91677a1/> added int, uint, and float types
 - <csr-id-592aaa39de6c785a735740c664f2c2fd19be13d9/> added number type as an alias to i64
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-b5fbe25d31673d4e8676883cdeee7166a5538da5/> ensured missing values for optional fields do not throw an error
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-22d92b58500869729edda0283123800557057ed3/> fixed sql component with multiple inputs, incorrect signature match, fixes #238, #239

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 28 commits contributed to the release over the course of 180 calendar days.
 - 182 days passed between releases.
 - 26 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#319](https://github.com/candlecorp/wick/issues/319), [#328](https://github.com/candlecorp/wick/issues/328), [#375](https://github.com/candlecorp/wick/issues/375)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#328](https://github.com/candlecorp/wick/issues/328)**
    - Added spread operator in SQL positional args, merge sql components. ([`cbf564e`](https://github.com/candlecorp/wick/commit/cbf564eebf5c96f1d827c319e927c5f4150c5e56))
 * **[#375](https://github.com/candlecorp/wick/issues/375)**
    - Fixed rustdoc, cleaned up buildability of individual crates ([`c3aae56`](https://github.com/candlecorp/wick/commit/c3aae5603084135101a302981dc6e72c9a257e8d))
 * **Uncategorized**
    - Updated versions and changelogs ([`35ff51b`](https://github.com/candlecorp/wick/commit/35ff51b8a93c27475765a7eb65c23256f4f93d67))
    - Migrated AsRef<str> to concrete types or Into<String> ([`60128f7`](https://github.com/candlecorp/wick/commit/60128f7707f2d2a537ffa32e24376f58d7faa7be))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Added more tests for #378, fixed fields being requide by default from config ([`ae1400c`](https://github.com/candlecorp/wick/commit/ae1400caa092433bec0f66c04bd6e0efea30d173))
    - Ensured missing values for optional fields do not throw an error ([`b5fbe25`](https://github.com/candlecorp/wick/commit/b5fbe25d31673d4e8676883cdeee7166a5538da5))
    - Added unions to type definitions ([`222cc7f`](https://github.com/candlecorp/wick/commit/222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f))
    - Added int, uint, and float types ([`8b8c990`](https://github.com/candlecorp/wick/commit/8b8c990bc76df17fad61c8fe903e64a3c91677a1))
    - Added number type as an alias to i64 ([`592aaa3`](https://github.com/candlecorp/wick/commit/592aaa39de6c785a735740c664f2c2fd19be13d9))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Fixed issue where component host would not report an accurate signature ([`495734d`](https://github.com/candlecorp/wick/commit/495734dc37a29801ca2c68c77da60d0b30905303))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Added wick new and better config serialization ([`f9a4b37`](https://github.com/candlecorp/wick/commit/f9a4b37da51df156e4293e639becbed06813ff46))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Added config validation, passing of config on command line, exposing config to user ([`954e9ff`](https://github.com/candlecorp/wick/commit/954e9ffbdab962ad051764f5a9dcb90bfe543175))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Added restapi router ([`58045d0`](https://github.com/candlecorp/wick/commit/58045d0fe75f519b84ebd45f3b1493e55fd4b282))
    - Fixed sql component with multiple inputs, incorrect signature match, fixes #238, #239 ([`22d92b5`](https://github.com/candlecorp/wick/commit/22d92b58500869729edda0283123800557057ed3))
    - Added proper type defs into config, closes #200. Fixed #228, #227 ([`49a53de`](https://github.com/candlecorp/wick/commit/49a53de6cb6631e2dc1f1e633d1c29d0510383cb))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added type imports ([`17c9058`](https://github.com/candlecorp/wick/commit/17c9058b98935fa8ed29dbc27b899c9e3244eb67))
    - Added the ability to create inline node IDs in flow config ([`f7d7274`](https://github.com/candlecorp/wick/commit/f7d72741adae67477634ccdf52b93fe8f0c3c35f))
</details>

## v0.16.0 (2023-09-14)

<csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/>
<csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/>
<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>

### Chore

 - <csr-id-60128f7707f2d2a537ffa32e24376f58d7faa7be/> migrated AsRef<str> to concrete types or Into<String>
 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog
 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-8b8c990bc76df17fad61c8fe903e64a3c91677a1/> added int, uint, and float types
 - <csr-id-592aaa39de6c785a735740c664f2c2fd19be13d9/> added number type as an alias to i64
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-b5fbe25d31673d4e8676883cdeee7166a5538da5/> ensured missing values for optional fields do not throw an error
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-22d92b58500869729edda0283123800557057ed3/> fixed sql component with multiple inputs, incorrect signature match, fixes #238, #239

## v0.15.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-222cc7f6b992f10ceeedfcf93b2d0b8b75d3de5f/> added unions to type definitions
 - <csr-id-cbf564eebf5c96f1d827c319e927c5f4150c5e56/> added spread operator in SQL positional args, merge sql components.
 - <csr-id-8b8c990bc76df17fad61c8fe903e64a3c91677a1/> added int, uint, and float types
 - <csr-id-592aaa39de6c785a735740c664f2c2fd19be13d9/> added number type as an alias to i64
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-f9a4b37da51df156e4293e639becbed06813ff46/> added wick new and better config serialization
 - <csr-id-954e9ffbdab962ad051764f5a9dcb90bfe543175/> added config validation, passing of config on command line, exposing config to user
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-58045d0fe75f519b84ebd45f3b1493e55fd4b282/> added restapi router
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports
 - <csr-id-f7d72741adae67477634ccdf52b93fe8f0c3c35f/> added the ability to create inline node IDs in flow config

### Bug Fixes

 - <csr-id-ae1400caa092433bec0f66c04bd6e0efea30d173/> added more tests for #378, fixed fields being requide by default from config
 - <csr-id-b5fbe25d31673d4e8676883cdeee7166a5538da5/> ensured missing values for optional fields do not throw an error
 - <csr-id-495734dc37a29801ca2c68c77da60d0b30905303/> fixed issue where component host would not report an accurate signature
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-22d92b58500869729edda0283123800557057ed3/> fixed sql component with multiple inputs, incorrect signature match, fixes #238, #239

## v0.15.0 (2023-04-18)

<csr-id-35047c3a741b00d88c4abc2ed3749af040a83671/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>

### Chore

 - <csr-id-35047c3a741b00d88c4abc2ed3749af040a83671/> release wick-xdg, wick-logger, asset-container, derive-asset-container, performance-mark, tap-harness, wick-interface-types, wick-packet

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-bc79d37c98b41e10815a9641396e73b3c4c3b55a/> added wick-test

### Bug Fixes

 - <csr-id-e7d2a9f088d1545bad308b1a95bfe8d2866ccefe/> fixed impl to consider signatures equal regardless of list order

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 19 calendar days.
 - 26 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release wick-xdg, wick-logger, asset-container, derive-asset-container, performance-mark, tap-harness, wick-interface-types, wick-packet ([`35047c3`](https://github.com/candlecorp/wick/commit/35047c3a741b00d88c4abc2ed3749af040a83671))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Fixed impl to consider signatures equal regardless of list order ([`e7d2a9f`](https://github.com/candlecorp/wick/commit/e7d2a9f088d1545bad308b1a95bfe8d2866ccefe))
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

### Chore

 - <csr-id-f229d8ee9dbb1c051d18b911bb4ef868b968ea14/> Release

### New Features

 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 2 calendar days.
 - 8 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ([`f229d8e`](https://github.com/candlecorp/wick/commit/f229d8ee9dbb1c051d18b911bb4ef868b968ea14))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
</details>

## v0.12.0 (2023-03-15)

<csr-id-c27131154861be5625b82e1e7d99d8228df1fa39/>

### New Features

 - <csr-id-8745221bb0e25332f85bebe2387bc10a440ed5ac/> added codegen based off component.yaml
 - <csr-id-97280ee71b361472dbb6ae32c77626b07c218554/> incorporated interface.json into component.yaml

### Other

 - <csr-id-c27131154861be5625b82e1e7d99d8228df1fa39/> added http types example as tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 4 calendar days.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/candlecorp/wick/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/candlecorp/wick/issues/144)**
    - Converted type maps to list ([`edd4a74`](https://github.com/candlecorp/wick/commit/edd4a7494bb638d95c49c4d40a042697a6da34c4))
 * **Uncategorized**
    - Fix: updated wick-component-codegen metadata fix: updated cargo deny configuration ([`51406ea`](https://github.com/candlecorp/wick/commit/51406ea741ef3d73389e3859c5a3ee41fba9079f))
    - Unified workspace dependencies, added versions ([`2f2c131`](https://github.com/candlecorp/wick/commit/2f2c13155e236a3d55d31adb2a12b5ea26e89f25))
    - Added codegen based off component.yaml ([`8745221`](https://github.com/candlecorp/wick/commit/8745221bb0e25332f85bebe2387bc10a440ed5ac))
    - Added http types example as tests ([`c271311`](https://github.com/candlecorp/wick/commit/c27131154861be5625b82e1e7d99d8228df1fa39))
    - Incorporated interface.json into component.yaml ([`97280ee`](https://github.com/candlecorp/wick/commit/97280ee71b361472dbb6ae32c77626b07c218554))
    - Renamed wasmflow->wick, migrated root-level tests to better locations ([`ed9bef3`](https://github.com/candlecorp/wick/commit/ed9bef306029db64675434500ba7c1519e65478e))
</details>

