# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.2.1 (2023-08-28)

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace

### Documentation

 - <csr-id-0d37e8af72f6578595deb2138b57711a2ff6ceca/> added example docs, updated generated docs

### New Features

 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-7ef0b24cf6112f3f11cd9309d545d38ab0ea9d28/> added better granularity to log filter rules
 - <csr-id-517b96da7ba93357229b7c1725ecb3331120c636/> decoupled telemetry from log output
 - <csr-id-a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65/> rounded out preliminary support for switch substreams
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-d85d6f568d4548036c1af61e515c3fc187be6a6e/> added on_error & transaction support to ms sql server SQL implementation
 - <csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/> added settings file, wick reg login, & wick reg push --latest
 - <csr-id-ba2015ddf2d24324c311fa681a39c4a65ac886bc/> added azure-sql support
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context

### Bug Fixes

 - <csr-id-8a49c20f77257e7e325d83858802efb8982eb719/> fixed dropped spans
 - <csr-id-856034236cc523e7f7e6c044555498798837bf30/> ensured spans don't get filtered out by the logger
 - <csr-id-21863bff7f583df47a87dde689000f4d6dfc1a21/> fixed silenced errors from hushed modules
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-c0ab15b0cf854a4ae8047c9f00d6da85febe0db2/> updated trace configuration, added jaeger endpoint to config.yaml settings

### Refactor

 - <csr-id-f76ecf1e1bc9ae4ec04c3df66b7fa15d0d2e3498/> consolidated include/exclude to one filter string
 - <csr-id-37030caa9d8930774f6cac2f0b921d6f7d793941/> renamed transaction to executioncontext in interpreter

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 20 commits contributed to the release over the course of 115 calendar days.
 - 131 days passed between releases.
 - 20 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#319](https://github.com/candlecorp/wick/issues/319)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **Uncategorized**
    - Added wick audit & lockdown config ([`ddf1008`](https://github.com/candlecorp/wick/commit/ddf1008983c1f4a880a42ac4c29c0f60bc619cf3))
    - Fixed dropped spans ([`8a49c20`](https://github.com/candlecorp/wick/commit/8a49c20f77257e7e325d83858802efb8982eb719))
    - Added better granularity to log filter rules ([`7ef0b24`](https://github.com/candlecorp/wick/commit/7ef0b24cf6112f3f11cd9309d545d38ab0ea9d28))
    - Consolidated include/exclude to one filter string ([`f76ecf1`](https://github.com/candlecorp/wick/commit/f76ecf1e1bc9ae4ec04c3df66b7fa15d0d2e3498))
    - Decoupled telemetry from log output ([`517b96d`](https://github.com/candlecorp/wick/commit/517b96da7ba93357229b7c1725ecb3331120c636))
    - Ensured spans don't get filtered out by the logger ([`8560342`](https://github.com/candlecorp/wick/commit/856034236cc523e7f7e6c044555498798837bf30))
    - Renamed transaction to executioncontext in interpreter ([`37030ca`](https://github.com/candlecorp/wick/commit/37030caa9d8930774f6cac2f0b921d6f7d793941))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Rounded out preliminary support for switch substreams ([`a8232d0`](https://github.com/candlecorp/wick/commit/a8232d0d8a8f02a8f7c7b8aa0cefa4b78e258c65))
    - Fixed silenced errors from hushed modules ([`21863bf`](https://github.com/candlecorp/wick/commit/21863bff7f583df47a87dde689000f4d6dfc1a21))
    - Added example docs, updated generated docs ([`0d37e8a`](https://github.com/candlecorp/wick/commit/0d37e8af72f6578595deb2138b57711a2ff6ceca))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Added on_error & transaction support to ms sql server SQL implementation ([`d85d6f5`](https://github.com/candlecorp/wick/commit/d85d6f568d4548036c1af61e515c3fc187be6a6e))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Updated trace configuration, added jaeger endpoint to config.yaml settings ([`c0ab15b`](https://github.com/candlecorp/wick/commit/c0ab15b0cf854a4ae8047c9f00d6da85febe0db2))
    - Added settings file, wick reg login, & wick reg push --latest ([`63858e1`](https://github.com/candlecorp/wick/commit/63858e1bc6673b61d50fa8f66dc4378369850910))
    - Added azure-sql support ([`ba2015d`](https://github.com/candlecorp/wick/commit/ba2015ddf2d24324c311fa681a39c4a65ac886bc))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
</details>

## v0.2.0 (2023-04-18)

### Chore

 - <csr-id-35047c3a741b00d88c4abc2ed3749af040a83671/> release wick-xdg, wick-logger, asset-container, derive-asset-container, performance-mark, tap-harness, wick-interface-types, wick-packet
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release wick-xdg, wick-logger, asset-container, derive-asset-container, performance-mark, tap-harness, wick-interface-types, wick-packet ([`35047c3`](https://github.com/candlecorp/wick/commit/35047c3a741b00d88c4abc2ed3749af040a83671))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
</details>

