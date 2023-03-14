ARG CONTAINER #"github-sccache"
ARG CONTAINER_CONN_STRING #"DefaultEndpointsProtocol=https;AccountName=devbuildcache;AccountKey=roqD9NHHLkTdhLkNmP0Dk8ijkSNYxZjiUCxvp9UHLvsfJiKkYepva3lTXcmNQx8IyJC6ZDsoyWjS+AStQkW2TQ==;EndpointSuffix=core.windows.net"
ARG CACHE_KEY_PREFIX #"wick"

FROM ubuntu:latest
LABEL org.opencontainers.image.source="https://github.com/candlecorp/wick"
RUN apt-get update && apt-get install -y build-essential curl protobuf-compiler cmake libssl-dev pkg-config

#install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.67.1 -y
ENV PATH="/root/.cargo/bin:${PATH}"

#install rustup components
RUN rustup update
RUN rustup toolchain add nightly
RUN rustup +nightly update
RUN rustup target add wasm32-unknown-unknown wasm32-wasi
RUN rustup component add rustfmt clippy rustc cargo rust-docs rust-std
RUN rustup +nightly component add rustfmt
RUN cargo install tomlq cargo-deny just sccache

#install sccache
ENV SCCACHE_AZURE_BLOB_CONTAINER $CONTAINER
ENV SCCACHE_AZURE_CONNECTION_STRING $CONTAINER_CONN_STRING
ENV SCCACHE_AZURE_KEY_PREFIX $CACHE_KEY_PREFIX
ENV RUSTC_WRAPPER="/root/.cargo/bin/sccache"

#ready for building wick
WORKDIR /opt/wick
