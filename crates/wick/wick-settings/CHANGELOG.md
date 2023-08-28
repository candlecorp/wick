# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0 (2023-08-28)

<csr-id-f76ecf1e1bc9ae4ec04c3df66b7fa15d0d2e3498/>

### New Features

 - <csr-id-517b96da7ba93357229b7c1725ecb3331120c636/> decoupled telemetry from log output
 - <csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/> added settings file, wick reg login, & wick reg push --latest

### Bug Fixes

 - <csr-id-3208691ffb824e9f83d9845ae274c9b60bb8d4fa/> converted all level spans to info_spans
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd/> fixed trace spans, jaeger->otlp, fixed serving static from root
 - <csr-id-c0ab15b0cf854a4ae8047c9f00d6da85febe0db2/> updated trace configuration, added jaeger endpoint to config.yaml settings

### Refactor

 - <csr-id-f76ecf1e1bc9ae4ec04c3df66b7fa15d0d2e3498/> consolidated include/exclude to one filter string

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 104 calendar days.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Consolidated include/exclude to one filter string ([`f76ecf1`](https://github.com/candlecorp/wick/commit/f76ecf1e1bc9ae4ec04c3df66b7fa15d0d2e3498))
    - Decoupled telemetry from log output ([`517b96d`](https://github.com/candlecorp/wick/commit/517b96da7ba93357229b7c1725ecb3331120c636))
    - Converted all level spans to info_spans ([`3208691`](https://github.com/candlecorp/wick/commit/3208691ffb824e9f83d9845ae274c9b60bb8d4fa))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Fixed trace spans, jaeger->otlp, fixed serving static from root ([`9cd1fc0`](https://github.com/candlecorp/wick/commit/9cd1fc007e6a21944f4fd65f3f65f4a2a86fd1bd))
    - Updated trace configuration, added jaeger endpoint to config.yaml settings ([`c0ab15b`](https://github.com/candlecorp/wick/commit/c0ab15b0cf854a4ae8047c9f00d6da85febe0db2))
    - Added settings file, wick reg login, & wick reg push --latest ([`63858e1`](https://github.com/candlecorp/wick/commit/63858e1bc6673b61d50fa8f66dc4378369850910))
</details>

