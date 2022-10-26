#!/usr/bin/env bash

# NanoBus location
: ${NANOBUS_INSTALL_DIR:="/usr/local/bin"}

# sudo is required to copy binary to NANOBUS_INSTALL_DIR for linux
: ${USE_SUDO:="false"}

# Http request CLI
NANOBUS_HTTP_REQUEST_CLI=curl

# GitHub Organization and repo name to download release
GITHUB_ORG=nanobus
GITHUB_REPO=nanobus

# NanoBus filename
NANOBUS_FILENAME=nanobus

NANOBUS_FILE="${NANOBUS_INSTALL_DIR}/${NANOBUS_FILENAME}"

NANOBUS_EXISTS=false

getSystemInfo() {
    ARCH=$(uname -m)
    case $ARCH in
        armv7*) ARCH="arm";;
        aarch64) ARCH="arm64";;
        x86_64) ARCH="amd64";;
    esac

    OS=$(echo `uname`|tr '[:upper:]' '[:lower:]')

    # Most linux distro needs root permission to copy the file to /usr/local/bin
    if [[ "$OS" == "linux" || "$OS" == "darwin" ]] && [ "$NANOBUS_INSTALL_DIR" == "/usr/local/bin" ]; then
        USE_SUDO="true"
    fi
}

verifySupported() {
    local supported=(darwin-amd64 darwin-arm64 linux-amd64 linux-arm)
    local current_osarch="${OS}-${ARCH}"

    for osarch in "${supported[@]}"; do
        if [ "$osarch" == "$current_osarch" ]; then
            echo "Your system is ${OS}_${ARCH}"
            return
        fi
    done

    echo "No prebuilt binary for ${current_osarch}"
    exit 1
}

runAsRoot() {
    local CMD="$*"

    if [ $EUID -ne 0 -a $USE_SUDO = "true" ]; then
        CMD="sudo $CMD"
    fi

    $CMD
}

checkHttpRequestCLI() {
    if type "curl" > /dev/null; then
        NANOBUS_HTTP_REQUEST_CLI=curl
    elif type "wget" > /dev/null; then
        NANOBUS_HTTP_REQUEST_CLI=wget
    else
        echo "Either curl or wget is required"
        exit 1
    fi
}

checkExistingNanoBus() {
    if [ -f "$NANOBUS_FILE" ]; then
        echo -e "\nNanoBus is detected:"
        $NANOBUS_FILE version
        echo -e "Reinstalling NanoBus - ${NANOBUS_FILE}...\n"
        NANOBUS_EXISTS=true
    else
        echo -e "Installing NanoBus...\n"
    fi
}

getLatestRelease() {
    local nanobusReleaseUrl="https://api.github.com/repos/${GITHUB_ORG}/${GITHUB_REPO}/releases"
    local latest_release=""

    if [ "$NANOBUS_HTTP_REQUEST_CLI" == "curl" ]; then
        latest_release=$(curl -s $nanobusReleaseUrl | grep \"tag_name\" | grep -v rc | awk 'NR==1{print $2}' |  sed -n 's/\"\(.*\)\",/\1/p')
    else
        latest_release=$(wget -q --header="Accept: application/json" -O - $nanobusReleaseUrl | grep \"tag_name\" | grep -v rc | awk 'NR==1{print $2}' |  sed -n 's/\"\(.*\)\",/\1/p')
    fi

    ret_val=$latest_release
}

downloadFile() {
    LATEST_RELEASE_TAG=$1

    NANOBUS_ARTIFACT="${NANOBUS_FILENAME}_${OS}_${ARCH}.tar.gz"
    DOWNLOAD_BASE="https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases/download"
    DOWNLOAD_URL="${DOWNLOAD_BASE}/${LATEST_RELEASE_TAG}/${NANOBUS_ARTIFACT}"

    # Create the temp directory
    NANOBUS_TMP_ROOT=$(mktemp -dt nanobus-install-XXXXXX)
    ARTIFACT_TMP_FILE="$NANOBUS_TMP_ROOT/$NANOBUS_ARTIFACT"

    echo "Downloading $DOWNLOAD_URL ..."
    if [ "$NANOBUS_HTTP_REQUEST_CLI" == "curl" ]; then
        curl -SsL "$DOWNLOAD_URL" -o "$ARTIFACT_TMP_FILE"
    else
        wget -q -O "$ARTIFACT_TMP_FILE" "$DOWNLOAD_URL"
    fi

    if [ ! -f "$ARTIFACT_TMP_FILE" ]; then
        echo "failed to download $DOWNLOAD_URL ..."
        exit 1
    fi
}

installFile() {
    tar xf "$ARTIFACT_TMP_FILE" -C "$NANOBUS_TMP_ROOT"
    local tmp_root_nanobus="$NANOBUS_TMP_ROOT/${NANOBUS_FILENAME}_${OS}_${ARCH}/$NANOBUS_FILENAME"

    if [ ! -f "$tmp_root_nanobus" ]; then
        echo "Failed to unpack NanoBus executable."
        exit 1
    fi

    chmod o+x $tmp_root_nanobus
    # Remove existing file to prevent signature caching.
    if [ "$NANOBUS_EXISTS" = true ]; then
        runAsRoot rm "$NANOBUS_INSTALL_DIR/$NANOBUS_FILENAME"    
    fi
    runAsRoot cp "$tmp_root_nanobus" "$NANOBUS_INSTALL_DIR"

    if [ -f "$NANOBUS_FILE" ]; then
        echo "$NANOBUS_FILENAME installed into $NANOBUS_INSTALL_DIR successfully."

        $NANOBUS_FILE version
    else 
        echo "Failed to install $NANOBUS_FILENAME"
        exit 1
    fi
}

fail_trap() {
    result=$?
    if [ "$result" != "0" ]; then
        echo "Failed to install NanoBus"
        echo "For support, go to https://nanobus.io"
    fi
    cleanup
    exit $result
}

cleanup() {
    if [[ -d "${NANOBUS_TMP_ROOT:-}" ]]; then
        rm -rf "$NANOBUS_TMP_ROOT"
    fi
}

installCompleted() {
    echo -e "\nNanoBus is installed successfully."
}

# -----------------------------------------------------------------------------
# main
# -----------------------------------------------------------------------------
trap "fail_trap" EXIT

getSystemInfo
verifySupported
checkExistingNanoBus
checkHttpRequestCLI


if [ -z "$1" ]; then
    echo "Getting the latest NanoBus..."
    getLatestRelease
else
    ret_val=v$1
fi

echo "Installing $ret_val NanoBus..."

downloadFile $ret_val
installFile
cleanup

installCompleted
