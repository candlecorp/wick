# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.0 (2023-09-14)

### Chore

 - <csr-id-7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c/> updated lints
 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/> fixed warnings

### Documentation

 - <csr-id-37905206a10ff16406b77ad296d467ebf76fc8fb/> added changelog
 - <csr-id-10672c5db34d10e50869b2c14977f9235761cabd/> updated config codegen, refactored config for clarity, fixed template

### New Features

 - <csr-id-398a034a3950c5b5dc95418248dfeb1f4f27f2bc/> added oci options so `wick reg pull` can pull more intuitively
 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-f4f04af492c7e0fe90472a6a5bafebfdbeddf622/> added types to package
 - <csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/> added settings file, wick reg login, & wick reg push --latest
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/> added context for wasm components
 - <csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/> added asset flags, fixed relative volumes, fixed manifest locations
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-947a6d9315cbfdcfd1e6780a47142b4273240b11/> wick run will run oci registry references
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/> adding tar.gz for extra files

### Bug Fixes

 - <csr-id-56a8c256db4b362f9298ca29ffd6d3b8577f88d2/> fixed broken push on package with registry deps
 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-d47da56a8cc73c32c39312e5a5ed58e8db5891d9/> remove println
 - <csr-id-46d29109dc6502ea826236cf5438c54e02674d04/> fixed package push trying to retrieve remote assets
 - <csr-id-cd2b609ec6f60ec4726440b7519b4d6149f3f664/> remove unused dependency
 - <csr-id-c61ca5f7320b533db3b69bdbe81fd37edbaa8eac/> multi-level recursive files in package
 - <csr-id-57698d4a6e4b86f5f438d12928ccdbbbb20a8abf/> fixed importing of assets from assets
 - <csr-id-a64d396dae1d8ed7c5cf4f21dba27eafb1294d0e/> recursive package assets
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 39 commits contributed to the release over the course of 135 calendar days.
 - 148 days passed between releases.
 - 37 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 5 unique issues were worked on: [#232](https://github.com/candlecorp/wick/issues/232), [#319](https://github.com/candlecorp/wick/issues/319), [#388](https://github.com/candlecorp/wick/issues/388), [#405](https://github.com/candlecorp/wick/issues/405), [#417](https://github.com/candlecorp/wick/issues/417)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#232](https://github.com/candlecorp/wick/issues/232)**
    - Added codec to HTTP server, added runtime constraints, ability to explicitly drop packets ([`1d37fb5`](https://github.com/candlecorp/wick/commit/1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71))
 * **[#319](https://github.com/candlecorp/wick/issues/319)**
    - Propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates ([`7ab25d2`](https://github.com/candlecorp/wick/commit/7ab25d2fc1274fbf552b86f59774b1b24ea12b0f))
 * **[#388](https://github.com/candlecorp/wick/issues/388)**
    - Added `wick install` ([`3158048`](https://github.com/candlecorp/wick/commit/3158048ad1d0c33518cb647d08f927606afcecd0))
 * **[#405](https://github.com/candlecorp/wick/issues/405)**
    - Fixed "refusing to overwrite ..." errors on application runs. ([`a10242d`](https://github.com/candlecorp/wick/commit/a10242d4786cfa199eaf61289b9da99d09c114a7))
 * **[#417](https://github.com/candlecorp/wick/issues/417)**
    - Fixed broken push on package with registry deps ([`56a8c25`](https://github.com/candlecorp/wick/commit/56a8c256db4b362f9298ca29ffd6d3b8577f88d2))
 * **Uncategorized**
    - Added oci options so `wick reg pull` can pull more intuitively ([`398a034`](https://github.com/candlecorp/wick/commit/398a034a3950c5b5dc95418248dfeb1f4f27f2bc))
    - Updated lints ([`7bb6865`](https://github.com/candlecorp/wick/commit/7bb686524f6adaaebbd3d6502ee24c0d5f6efc7c))
    - Merge remote-tracking branch 'refs/remotes/origin/main' ([`344b60c`](https://github.com/candlecorp/wick/commit/344b60c854bd33f1d267c7f422378e2716496ba6))
    - Added changelog ([`3790520`](https://github.com/candlecorp/wick/commit/37905206a10ff16406b77ad296d467ebf76fc8fb))
    - Made v0, v1, and normalized config conditional features ([`f28522f`](https://github.com/candlecorp/wick/commit/f28522fa663f121f5da90df9dd8461d85c6222ed))
    - Added wick audit & lockdown config ([`ddf1008`](https://github.com/candlecorp/wick/commit/ddf1008983c1f4a880a42ac4c29c0f60bc619cf3))
    - Explicitly defined all features ([`7968fb0`](https://github.com/candlecorp/wick/commit/7968fb0b6fe519732595ed1e3ed9cc429a45d0c4))
    - Updated include directives in Cargo.toml ([`4090c8f`](https://github.com/candlecorp/wick/commit/4090c8fa7fba8254570cc10024fd8a6b15c076ab))
    - Eliminated fetching of bytes before checking cache ([`586ace0`](https://github.com/candlecorp/wick/commit/586ace0978ca8adf58bf4d1fa5ed392015297c21))
    - Fixed included cached assets on wick reg push ([`4577461`](https://github.com/candlecorp/wick/commit/4577461e0a767ec99ae6482c2e2efeb3069ca0c8))
    - Made configuration init a hard boundary with earlier validation ([`bf23983`](https://github.com/candlecorp/wick/commit/bf239832ccb282b7ce56430157a3412efc9737a6))
    - Made env path usage more clear, fixed pull behavior, added wick show ([`fac116c`](https://github.com/candlecorp/wick/commit/fac116c0a98235e454dfdd4826e11508ebae68c6))
    - Changed formal datetime type to DateTime<Utc> ([`f113d30`](https://github.com/candlecorp/wick/commit/f113d307535081caa4248315607db17f3180a107))
    - Added support for input-less ops, added test for wasm RNG from inherent seed ([`3213e75`](https://github.com/candlecorp/wick/commit/3213e75c9e1a08db300d521e228d65e27671a779))
    - Made inherent data required vs optional/missing ([`221be20`](https://github.com/candlecorp/wick/commit/221be200017943aae5d2c78254a8194d72600f7a))
    - Updated config codegen, refactored config for clarity, fixed template ([`10672c5`](https://github.com/candlecorp/wick/commit/10672c5db34d10e50869b2c14977f9235761cabd))
    - Removed unused deps, consolidated versions at root workspace ([`e452ae3`](https://github.com/candlecorp/wick/commit/e452ae37b04b13666129fcbaa4af089555d456a2))
    - Fixed warnings ([`ab7d535`](https://github.com/candlecorp/wick/commit/ab7d5355945adb592c4e00ccdc8b268e146e6535))
    - Remove println ([`d47da56`](https://github.com/candlecorp/wick/commit/d47da56a8cc73c32c39312e5a5ed58e8db5891d9))
    - Added types to package ([`f4f04af`](https://github.com/candlecorp/wick/commit/f4f04af492c7e0fe90472a6a5bafebfdbeddf622))
    - Fixed package push trying to retrieve remote assets ([`46d2910`](https://github.com/candlecorp/wick/commit/46d29109dc6502ea826236cf5438c54e02674d04))
    - Remove unused dependency ([`cd2b609`](https://github.com/candlecorp/wick/commit/cd2b609ec6f60ec4726440b7519b4d6149f3f664))
    - Multi-level recursive files in package ([`c61ca5f`](https://github.com/candlecorp/wick/commit/c61ca5f7320b533db3b69bdbe81fd37edbaa8eac))
    - Fixed importing of assets from assets ([`57698d4`](https://github.com/candlecorp/wick/commit/57698d4a6e4b86f5f438d12928ccdbbbb20a8abf))
    - Recursive package assets ([`a64d396`](https://github.com/candlecorp/wick/commit/a64d396dae1d8ed7c5cf4f21dba27eafb1294d0e))
    - Added settings file, wick reg login, & wick reg push --latest ([`63858e1`](https://github.com/candlecorp/wick/commit/63858e1bc6673b61d50fa8f66dc4378369850910))
    - Normalized accessor api for wick-config ([`56959c7`](https://github.com/candlecorp/wick/commit/56959c74e0fa96870d6fdd4197a30606041a0f8a))
    - Added context for wasm components ([`27c1fba`](https://github.com/candlecorp/wick/commit/27c1fba1d6af314e3b5f317178426331acc4b071))
    - Added asset flags, fixed relative volumes, fixed manifest locations ([`3dd4cdb`](https://github.com/candlecorp/wick/commit/3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08))
    - Added glob support in package files ([`53ff1dd`](https://github.com/candlecorp/wick/commit/53ff1dd49057a0b7cb45deff02b350d8f1b2970e))
    - Fixed linting issues ([`6c6f9a8`](https://github.com/candlecorp/wick/commit/6c6f9a80f9873f5989453c7800a355724cb61fff))
    - Wick run will run oci registry references ([`947a6d9`](https://github.com/candlecorp/wick/commit/947a6d9315cbfdcfd1e6780a47142b4273240b11))
    - Added gzip error handling ([`6fb111c`](https://github.com/candlecorp/wick/commit/6fb111cc0068ca5a4709ef274b046c0b590eee08))
    - Adding tar.gz for extra files ([`8c58c35`](https://github.com/candlecorp/wick/commit/8c58c354e765a51abb602b184c45055b9d561ed5))
</details>

## v0.2.1 (2023-08-28)

<csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/>
<csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/>
<csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/>
<csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/>
<csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/>
<csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/>
<csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/>

### Chore

 - <csr-id-7968fb0b6fe519732595ed1e3ed9cc429a45d0c4/> explicitly defined all features
 - <csr-id-4090c8fa7fba8254570cc10024fd8a6b15c076ab/> updated include directives in Cargo.toml
 - <csr-id-e452ae37b04b13666129fcbaa4af089555d456a2/> removed unused deps, consolidated versions at root workspace
 - <csr-id-ab7d5355945adb592c4e00ccdc8b268e146e6535/> fixed warnings

### Documentation

 - <csr-id-10672c5db34d10e50869b2c14977f9235761cabd/> updated config codegen, refactored config for clarity, fixed template

### New Features

 - <csr-id-ddf1008983c1f4a880a42ac4c29c0f60bc619cf3/> added wick audit & lockdown config
 - <csr-id-7ab25d2fc1274fbf552b86f59774b1b24ea12b0f/> propagated context to non-wasm components, removed $ENV syntax in favor of liquid templates
 - <csr-id-3213e75c9e1a08db300d521e228d65e27671a779/> added support for input-less ops, added test for wasm RNG from inherent seed
 - <csr-id-f4f04af492c7e0fe90472a6a5bafebfdbeddf622/> added types to package
 - <csr-id-63858e1bc6673b61d50fa8f66dc4378369850910/> added settings file, wick reg login, & wick reg push --latest
 - <csr-id-56959c74e0fa96870d6fdd4197a30606041a0f8a/> normalized accessor api for wick-config
 - <csr-id-1d37fb5a9aebec3653425ddc102c2f2d4f5fcd71/> added codec to HTTP server, added runtime constraints, ability to explicitly drop packets
 - <csr-id-27c1fba1d6af314e3b5f317178426331acc4b071/> added context for wasm components
 - <csr-id-3dd4cdb6ff02a5ccdeb32d28522a8a0fe24e3d08/> added asset flags, fixed relative volumes, fixed manifest locations
 - <csr-id-53ff1dd49057a0b7cb45deff02b350d8f1b2970e/> added glob support in package files
 - <csr-id-947a6d9315cbfdcfd1e6780a47142b4273240b11/> wick run will run oci registry references
 - <csr-id-6fb111cc0068ca5a4709ef274b046c0b590eee08/> added gzip error handling
 - <csr-id-8c58c354e765a51abb602b184c45055b9d561ed5/> adding tar.gz for extra files

### Bug Fixes

 - <csr-id-56a8c256db4b362f9298ca29ffd6d3b8577f88d2/> fixed broken push on package with registry deps
 - <csr-id-4577461e0a767ec99ae6482c2e2efeb3069ca0c8/> fixed included cached assets on wick reg push
 - <csr-id-bf239832ccb282b7ce56430157a3412efc9737a6/> made configuration init a hard boundary with earlier validation
 - <csr-id-fac116c0a98235e454dfdd4826e11508ebae68c6/> made env path usage more clear, fixed pull behavior, added wick show
 - <csr-id-f113d307535081caa4248315607db17f3180a107/> changed formal datetime type to DateTime<Utc>
 - <csr-id-221be200017943aae5d2c78254a8194d72600f7a/> made inherent data required vs optional/missing
 - <csr-id-d47da56a8cc73c32c39312e5a5ed58e8db5891d9/> remove println
 - <csr-id-46d29109dc6502ea826236cf5438c54e02674d04/> fixed package push trying to retrieve remote assets
 - <csr-id-cd2b609ec6f60ec4726440b7519b4d6149f3f664/> remove unused dependency
 - <csr-id-c61ca5f7320b533db3b69bdbe81fd37edbaa8eac/> multi-level recursive files in package
 - <csr-id-57698d4a6e4b86f5f438d12928ccdbbbb20a8abf/> fixed importing of assets from assets
 - <csr-id-a64d396dae1d8ed7c5cf4f21dba27eafb1294d0e/> recursive package assets
 - <csr-id-6c6f9a80f9873f5989453c7800a355724cb61fff/> fixed linting issues

### Other

 - <csr-id-3158048ad1d0c33518cb647d08f927606afcecd0/> Added `wick install`
   * feat: added `wick install`
   
   * fix: using batch and ps1 files vs links on windows
   
   * ci: increment wick version
   
   * test: added wick install test for local app
   
   ---------

### Refactor

 - <csr-id-f28522fa663f121f5da90df9dd8461d85c6222ed/> made v0, v1, and normalized config conditional features
 - <csr-id-586ace0978ca8adf58bf4d1fa5ed392015297c21/> eliminated fetching of bytes before checking cache

## v0.2.0 (2023-04-19)

<csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/>
<csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/>
<csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/>
<csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/>
<csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/>
<csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/>
<csr-id-7e2538202a03999c2b5781d7658b72118dce9446/>
<csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/>
<csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/>

### Chore

 - <csr-id-1279be06f6cf8bc91641be7ab48d7941819c98fe/> release wick-cli and rest of crates
 - <csr-id-82fd51f5f813ea6887f40a0df031f33e13b0fd99/> removing unused dependencies
 - <csr-id-45c7b192ab740c7b1c0f60466e73e3f6cb9d21be/> renamed some packages to be unique for crates.io
 - <csr-id-f7c7615186d900b8f509355b2012dec66c4ad76a/> added missing metadata in Cargo.toml projects
 - <csr-id-fd3bedfb6b847ad5fe19d0838443cc308d75ab2b/> cleaned up comments, errors, et al

### New Features

 - <csr-id-b37172df9032fcd8a63f0a10552c12206f4c5518/> application is downloaded to current dir
 - <csr-id-3ebf4f195d0714839beef5b1620913aac9508989/> working registry push and pull
 - <csr-id-10335669483d0498968cdabe194e11d6c4907c19/> integrated config and asset creates
 - <csr-id-559b0370efb26403885fecb914efcea1cfcbc7e0/> create wick-package crate for uploading to oci

### Bug Fixes

 - <csr-id-1c58123f86ec95073b503790fe272b04003a05df/> adjusted default features on deps
 - <csr-id-4947e4665e1decc34f226ad0c116b8259c7c2153/> improved error handling of wick-package

### Refactor

 - <csr-id-ce7bc3a3ff467aa8834301697daca0398c61222c/> reorganized config to make further additions sustainable
 - <csr-id-7e2538202a03999c2b5781d7658b72118dce9446/> pulled package-related OCI methods into wick-oci-utils

### Test

 - <csr-id-ce40e430c0aae30ef85a710f5476d32a87d4dec4/> added postgres and mssql to integration setup
 - <csr-id-3802bf93746725527d5dfa80f3c65d3314d4122c/> added registry tests, invoke tests, v1 tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 15 commits contributed to the release over the course of 14 calendar days.
 - 15 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release wick-cli and rest of crates ([`1279be0`](https://github.com/candlecorp/wick/commit/1279be06f6cf8bc91641be7ab48d7941819c98fe))
    - Removing unused dependencies ([`82fd51f`](https://github.com/candlecorp/wick/commit/82fd51f5f813ea6887f40a0df031f33e13b0fd99))
    - Renamed some packages to be unique for crates.io ([`45c7b19`](https://github.com/candlecorp/wick/commit/45c7b192ab740c7b1c0f60466e73e3f6cb9d21be))
    - Added missing metadata in Cargo.toml projects ([`f7c7615`](https://github.com/candlecorp/wick/commit/f7c7615186d900b8f509355b2012dec66c4ad76a))
    - Added postgres and mssql to integration setup ([`ce40e43`](https://github.com/candlecorp/wick/commit/ce40e430c0aae30ef85a710f5476d32a87d4dec4))
    - Reorganized config to make further additions sustainable ([`ce7bc3a`](https://github.com/candlecorp/wick/commit/ce7bc3a3ff467aa8834301697daca0398c61222c))
    - Adjusted default features on deps ([`1c58123`](https://github.com/candlecorp/wick/commit/1c58123f86ec95073b503790fe272b04003a05df))
    - Cleaned up comments, errors, et al ([`fd3bedf`](https://github.com/candlecorp/wick/commit/fd3bedfb6b847ad5fe19d0838443cc308d75ab2b))
    - Added registry tests, invoke tests, v1 tests ([`3802bf9`](https://github.com/candlecorp/wick/commit/3802bf93746725527d5dfa80f3c65d3314d4122c))
    - Pulled package-related OCI methods into wick-oci-utils ([`7e25382`](https://github.com/candlecorp/wick/commit/7e2538202a03999c2b5781d7658b72118dce9446))
    - Improved error handling of wick-package ([`4947e46`](https://github.com/candlecorp/wick/commit/4947e4665e1decc34f226ad0c116b8259c7c2153))
    - Application is downloaded to current dir ([`b37172d`](https://github.com/candlecorp/wick/commit/b37172df9032fcd8a63f0a10552c12206f4c5518))
    - Working registry push and pull ([`3ebf4f1`](https://github.com/candlecorp/wick/commit/3ebf4f195d0714839beef5b1620913aac9508989))
    - Integrated config and asset creates ([`1033566`](https://github.com/candlecorp/wick/commit/10335669483d0498968cdabe194e11d6c4907c19))
    - Create wick-package crate for uploading to oci ([`559b037`](https://github.com/candlecorp/wick/commit/559b0370efb26403885fecb914efcea1cfcbc7e0))
</details>

