# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.0 (2023-08-28)

<csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/>

### Chore

 - <csr-id-eb26a1586f0e00137bbd9ee608cd15d3cde074d0/> updated lints, deprecated Link type, removed Ref type, renamed Custom->Named

### Chore

 - <csr-id-e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa/> generated changelogs

### Documentation

 - <csr-id-10672c5db34d10e50869b2c14977f9235761cabd/> updated config codegen, refactored config for clarity, fixed template

### New Features

 - <csr-id-dd57e5062f3cf5d01e163ad104e56f7debc50aa4/> added xml codec for wick-http-component
 - <csr-id-85e1abfc142a4f20e12a498e68c83de3f9971e8f/> added request/response middle to http trigger, refactored component codegen
 - <csr-id-49a53de6cb6631e2dc1f1e633d1c29d0510383cb/> added proper type defs into config, closes #200. Fixed #228, #227
 - <csr-id-88dbedb624e1e381f253fb6b56d9af81ceeb00c8/> added operation context
 - <csr-id-027392a9514ba4846e068b21476e980ea53bee1d/> added pluck & merge
 - <csr-id-dbbd787131fd959c8cf5c8130ca03da6a63221e7/> added http client component
 - <csr-id-17c9058b98935fa8ed29dbc27b899c9e3244eb67/> added type imports

### Bug Fixes

 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>

### New Features (BREAKING)

 - <csr-id-34e1484443de014ebe010063640f937e528df10a/> changed pre-request middleware to one output union vs a request/response race

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release over the course of 126 calendar days.
 - 131 days passed between releases.
 - 13 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Generated changelogs ([`e1d6c05`](https://github.com/candlecorp/wick/commit/e1d6c0542a79afd4b67fb1cf1d9ba87421302dfa))
    - Added xml codec for wick-http-component ([`dd57e50`](https://github.com/candlecorp/wick/commit/dd57e5062f3cf5d01e163ad104e56f7debc50aa4))
    - Changed pre-request middleware to one output union vs a request/response race ([`34e1484`](https://github.com/candlecorp/wick/commit/34e1484443de014ebe010063640f937e528df10a))
    - Fixed included cached assets on wick reg push ([`4577461`](https://github.com/candlecorp/wick/commit/4577461e0a767ec99ae6482c2e2efeb3069ca0c8))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Updated config codegen, refactored config for clarity, fixed template ([`10672c5`](https://github.com/candlecorp/wick/commit/10672c5db34d10e50869b2c14977f9235761cabd))
    - Updated lints, deprecated Link type, removed Ref type, renamed Custom->Named ([`eb26a15`](https://github.com/candlecorp/wick/commit/eb26a1586f0e00137bbd9ee608cd15d3cde074d0))
    - Added request/response middle to http trigger, refactored component codegen ([`85e1abf`](https://github.com/candlecorp/wick/commit/85e1abfc142a4f20e12a498e68c83de3f9971e8f))
    - Added proper type defs into config, closes #200. Fixed #228, #227 ([`49a53de`](https://github.com/candlecorp/wick/commit/49a53de6cb6631e2dc1f1e633d1c29d0510383cb))
    - Added operation context ([`88dbedb`](https://github.com/candlecorp/wick/commit/88dbedb624e1e381f253fb6b56d9af81ceeb00c8))
    - Added pluck & merge ([`027392a`](https://github.com/candlecorp/wick/commit/027392a9514ba4846e068b21476e980ea53bee1d))
    - Added http client component ([`dbbd787`](https://github.com/candlecorp/wick/commit/dbbd787131fd959c8cf5c8130ca03da6a63221e7))
    - Added type imports ([`17c9058`](https://github.com/candlecorp/wick/commit/17c9058b98935fa8ed29dbc27b899c9e3244eb67))
</details>

## v0.2.0 (2023-04-19)

<csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/>
<csr-id-406c10999648ca923fc8994b5835d11c823c19ce/>
<csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/>

### Chore

 - <csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/> release wick-cli and rest of crates
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al
 - <csr-id-406c10999648ca923fc8994b5835d11c823c19ce/> more renaming fixes + better error messages

### New Features

 - <csr-id-73e631097656436f10eda91816c137fa94c1a043/> added generated code to wrap responses
 - <csr-id-39fb923c30ec819bcbe665ef4fad569eebdfe194/> substreams/bracketing + codegen improvements
 - <csr-id-d90f0ab4aa1afc911859d2877903bc1f164cfbf5/> added http trigger

### Bug Fixes

 - <csr-id-5c807f221fbb2eefaedaa899f82da3e8f2600388/> fixed broken tests

### Refactor

 - <csr-id-fd71df4baaa3f856454624396eff9d9ee8c4473f/> centralized APIs around configuration yaml

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 29 calendar days.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release wick-cli and rest of crates ([`1279be0`](https://github.com/candlecorp/wick/commit/1279be06f6cf8bc91641be7ab48d7941819c98fe))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added generated code to wrap responses ([`73e6310`](https://github.com/candlecorp/wick/commit/73e631097656436f10eda91816c137fa94c1a043))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Centralized APIs around configuration yaml ([`fd71df4`](https://github.com/candlecorp/wick/commit/fd71df4baaa3f856454624396eff9d9ee8c4473f))
    - Fixed broken tests ([`5c807f2`](https://github.com/candlecorp/wick/commit/5c807f221fbb2eefaedaa899f82da3e8f2600388))
    - Substreams/bracketing + codegen improvements ([`39fb923`](https://github.com/candlecorp/wick/commit/39fb923c30ec819bcbe665ef4fad569eebdfe194))
    - More renaming fixes + better error messages ([`406c109`](https://github.com/candlecorp/wick/commit/406c10999648ca923fc8994b5835d11c823c19ce))
    - Added http trigger ([`d90f0ab`](https://github.com/candlecorp/wick/commit/d90f0ab4aa1afc911859d2877903bc1f164cfbf5))
</details>

