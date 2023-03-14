FROM ubuntu:latest
LABEL org.opencontainers.image.source="https://github.com/candlecorp/wick"
RUN apt-get update && apt-get install -y build-essential curl protobuf-compiler cmake libssl-dev pkg-config gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross
# Install cross-compilation toolchains for macOS and Windows
RUN apt-get install -y \
    llvm \
    clang \
    lldb \
    lld \
    mingw-w64

#install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.67.1 -y
ENV PATH="/root/.cargo/bin:${PATH}"

#install rustup components
RUN rustup update
RUN rustup toolchain add nightly
RUN rustup +nightly update
RUN rustup target add wasm32-unknown-unknown wasm32-wasi x86_64-unknown-linux-gnu i686-unknown-linux-gnu x86_64-pc-windows-gnu aarch64-unknown-linux-gnu aarch64-apple-darwin x86_64-apple-darwin
RUN rustup component add rustfmt clippy rustc cargo rust-docs rust-std
RUN rustup +nightly component add rustfmt
RUN cargo install tomlq cargo-deny just sccache

#set environment variables for arch64 cross-compilation
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

# Set environment variables for macOS cross-compilation
ENV CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=x86_64-apple-darwin20.6.0-ld

# Set environment variables for Windows cross-compilation
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
ENV CARGO_TARGET_I686_PC_WINDOWS_GNU_LINKER=i686-w64-mingw32-gcc

WORKDIR /opt/wick