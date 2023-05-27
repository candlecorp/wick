---
title: Installation
---
## Express Installation

Mac and Linux users can install the latest stable version of Wick with the following command in terminal:

```bash
curl -sSL sh.wick.run | bash
```

To download and install the nightly version, or other releases of wick, pass a single argument of the desired release.
```bash
curl -sSL sh.wick.run | bash -s -- nightly
```
Quick Install - Windows

Windows users can install the latest stable version of Wick with the following command in powershell:
```powershell
curl https://ps.wick.run -UseBasicParsing | Invoke-Expression
```
To download and install the nightly version, or other releases of wick, pass a single argument of the desired release.
```powershell
curl https://ps.wick.run -OutFile setup-wick.ps1; .\setup-wick.ps1 -ReleaseVersion "nightly"; rm setup-wick.ps1;
```

## Manual Install
You can build using the [Manual Install](https://github.com/candlecorp/wick#manual-install) instructions from the main repository.