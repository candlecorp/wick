<#
.SYNOPSIS
    Installs the wick project.
.PARAMETER ReleaseVersion
    The release version to install. Defaults to "latest".
#>
param (
    [string]$ReleaseVersion = "latest"
)

$ErrorActionPreference = "Stop"

$OrgName = "candlecorp"
$ProjectName = "wick"

$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:USERPROFILE\.wick\bin" }
$ArchiveFiles = @("wick.exe")
$InstalledTestBin = Join-Path $InstallDir $ArchiveFiles[0]

$SupportMessage = "For support, go to https://candle.dev/, join our Discord https://discord.gg/candle or reach out to us on Twitter @candle_corp."
$BaseUrl = "https://github.com/$OrgName/$ProjectName/releases"

$TmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())


$ArtifactName = "$ProjectName-win-amd64.zip"
$ArtifactTmpFile = Join-Path $TmpRoot $ArtifactName

$IntroMsg = "This will install $ProjectName to $InstallDir."

function Download-File {
    param (
        [string]$ReleaseTag
    )
    $DownloadUrl = "$BaseUrl/download/$ReleaseTag/$ArtifactName"

    if ($ReleaseTag -eq "latest") {
        $DownloadUrl = "$BaseUrl/$ReleaseTag/download/$ArtifactName"
    }

    # Create TmpRoot
    if (-not (Test-Path $TmpRoot)) {
        New-Item -ItemType Directory -Path $TmpRoot
    }

    Write-Host "Downloading $DownloadUrl..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArtifactTmpFile
}

function Install-File {
    Expand-Archive -Path $ArtifactTmpFile -DestinationPath $TmpRoot

    Write-Host "Copying files to $InstallDir"
    foreach ($file in $ArchiveFiles) {
        $filePath = Join-Path $TmpRoot $file
        $destPath = Join-Path $InstallDir $file

        if (-not (Test-Path $filePath)) {
            Write-Host "Failed to unpack $filePath."
            exit 3
        }

        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir | Out-Null
        }

        Copy-Item -Path $filePath -Destination $destPath

        if (Test-Path $destPath) {
            Write-Host "Installed $file into $InstallDir successfully"
        } else {
            Write-Host "Could not find $destPath, installation failed."
            exit 4
        }
    }
}

function Add-Path {
    $ProfilePath = $PROFILE

    $ParentDir = Split-Path $ProfilePath -Parent
    if (-not (Test-Path $ParentDir)) {
        New-Item -ItemType Directory -Path $ParentDir | Out-Null
    }

    if (-not (Test-Path $ProfilePath)) {
        New-Item -ItemType File -Path $ProfilePath | Out-Null
    }

    $CurrentProfileContent = Get-Content -Path $ProfilePath -Raw

    if (-not ($CurrentProfileContent -match [regex]::Escape($InstallDir))) {
        $PathUpdateLine = "`n`$env:PATH += `";$InstallDir`""
        Add-Content -Path $ProfilePath -Value $PathUpdateLine
        Write-Host "Updated PATH variable in $ProfilePath"
        Write-Host "Please open a new PowerShell session to start using $ProjectName"
    } else {
        Write-Host "$InstallDir is already in PATH"
    }
}

function Create-ConfigFile {
    $configDir = "$env:USERPROFILE\.wick\config"
    $configFile = Join-Path $configDir "config.yaml"

    if (-not (Test-Path $configFile)) {
        Write-Host "Creating $configFile..."
        if (-not (Test-Path $configDir)) {
            New-Item -ItemType Directory -Path $configDir | Out-Null
        }
        New-Item -ItemType File -Path $configFile | Out-Null
    } else {
        Write-Host "$configFile already exists."
    }
}

function Install-Completed {
    Write-Host "**** Congratulations, $ProjectName installed successfully! ****"
}

Write-Host "Installing $ProjectName $ReleaseVersion"

Write-Host $IntroMsg
Download-File -ReleaseTag $ReleaseVersion
Install-File
Remove-Item -Path $TmpRoot -Recurse -Force
Add-Path
Create-ConfigFile
Install-Completed