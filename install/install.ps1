param (
    [string]$Version,
    [string]$nanobusRoot = "C:\nanobus"
)

Write-Output ""
$ErrorActionPreference = 'stop'

#Escape space of nanobusRoot path
$nanobusRoot = $nanobusRoot -replace ' ', '` '

# Constants
$nanobusFileName = "nanobus.exe"
$nanobusFilePath = "${nanobusRoot}\${nanobusFileName}"
$nanobusZipExtracted = "${nanobusRoot}\nanobus_windows_amd64"

# GitHub Org and repo hosting nanobus CLI
$GitHubOrg = "nanobus"
$GitHubRepo = "nanobus"

# Set Github request authentication for basic authentication.
if ($Env:GITHUB_USER) {
    $basicAuth = [System.Convert]::ToBase64String([System.Text.Encoding]::ASCII.GetBytes($Env:GITHUB_USER + ":" + $Env:GITHUB_TOKEN));
    $githubHeader = @{"Authorization" = "Basic $basicAuth" }
}
else {
    $githubHeader = @{}
}

if ((Get-ExecutionPolicy) -gt 'RemoteSigned' -or (Get-ExecutionPolicy) -eq 'ByPass') {
    Write-Output "PowerShell requires an execution policy of 'RemoteSigned'."
    Write-Output "To make this change please run:"
    Write-Output "'Set-ExecutionPolicy RemoteSigned -scope CurrentUser'"
    break
}

# Change security protocol to support TLS 1.2 / 1.1 / 1.0 - old powershell uses TLS 1.0 as a default protocol
[Net.ServicePointManager]::SecurityProtocol = "tls12, tls11, tls"

# Check if nanobus CLI is installed.
if (Test-Path $nanobusFilePath -PathType Leaf) {
    Write-Warning "NanoBus is detected - $nanobusFilePath"
    Invoke-Expression "$nanobusFilePath version"
    Write-Output "Reinstalling NanoBus..."
}
else {
    Write-Output "Installing NanoBus..."
}

# Create nanobus Directory
Write-Output "Creating $nanobusRoot directory"
New-Item -ErrorAction Ignore -Path $nanobusRoot -ItemType "directory"
if (!(Test-Path $nanobusRoot -PathType Container)) {
    throw "Cannot create $nanobusRoot"
}

# Get the list of release from GitHub
$releases = Invoke-RestMethod -Headers $githubHeader -Uri "https://api.github.com/repos/${GitHubOrg}/${GitHubRepo}/releases" -Method Get
if ($releases.Count -eq 0) {
    throw "No releases from github.com/nanobus/cli repo"
}

# Filter windows binary and download archive
if (!$Version) {
    $windowsAsset = $releases | Where-Object { $_.tag_name -notlike "*rc*" } | Select-Object -First 1 | Select-Object -ExpandProperty assets | Where-Object { $_.name -Like "*windows_amd64.zip" }
    if (!$windowsAsset) {
        throw "Cannot find the windows nanobus CLI binary"
    }
    $zipFileUrl = $windowsAsset.url
    $assetName = $windowsAsset.name
} else {
    $assetName = "nanobus_windows_amd64.zip"
    $zipFileUrl = "https://github.com/${GitHubOrg}/${GitHubRepo}/releases/download/v${Version}/${assetName}"
}

$zipFilePath = $nanobusRoot + "\" + $assetName
Write-Output "Downloading $zipFileUrl ..."

$githubHeader.Accept = "application/octet-stream"
Invoke-WebRequest -Headers $githubHeader -Uri $zipFileUrl -OutFile $zipFilePath
if (!(Test-Path $zipFilePath -PathType Leaf)) {
    throw "Failed to download NanoBus binary - $zipFilePath"
}

# Extract nanobus CLI to $nanobusRoot
Write-Output "Extracting $zipFilePath..."
Microsoft.Powershell.Archive\Expand-Archive -Force -Path $zipFilePath -DestinationPath $nanobusRoot
if (!(Test-Path $nanobusZipExtracted -PathType Container)) {
    throw "Failed to extract NanoBus archieve - $nanobusZipExtracted"
}

Copy-Item $nanobusZipExtracted\${nanobusFileName} -Destination $nanobusRoot
Remove-Item $nanobusZipExtracted -Force -Recurse

# Clean up zipfile
Write-Output "Clean up $zipFilePath..."
Remove-Item $zipFilePath -Force

# Add nanobusRoot directory to User Path environment variable
Write-Output "Try to add $nanobusRoot to User Path Environment variable..."
$UserPathEnvironmentVar = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPathEnvironmentVar -like '*nanobus*') {
    Write-Output "Skipping to add $nanobusRoot to User Path - $UserPathEnvironmentVar"
}
else {
    [System.Environment]::SetEnvironmentVariable("PATH", $UserPathEnvironmentVar + ";$nanobusRoot", "User")
    $UserPathEnvironmentVar = [Environment]::GetEnvironmentVariable("PATH", "User")
    Write-Output "Added $nanobusRoot to User Path - $UserPathEnvironmentVar"
}

# Check the nanobus CLI version
Invoke-Expression "$nanobusFilePath version"

Write-Output "`r`nNanoBus is installed successfully."
Write-Output "`r`nYou will need to start a new shell for the updated PATH."
