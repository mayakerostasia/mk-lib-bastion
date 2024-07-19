####################################################################################################
## Builder
####################################################################################################
FROM rust:1.78.0 AS builder
SHELL ["/bin/bash", "-c"]
WORKDIR /app
COPY . .
RUN apt-get update && DEBIAN_FRONTEND=noninteractive \
    apt-get install --yes \
    libcurl4 \
    build-essential \
    cmake \
    clang \
    && apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN --mount=type=ssh cargo build --release

####################################################################################################
## Final images
####################################################################################################

####################################################################################################
# Asset
####################################################################################################
FROM ghcr.io/bluebastion/dev-bb-rs-builder:v0.1.0 as asset
ARG RUST_BIN="bb-bin-monkey"
ENV RUST_BIN $RUST_BIN
ENV RUST_LOG "debug"
ENV CARGO_PKG_NAME "bb-monkey"
ENV SERVICE_NAME "bb-monkey"
# Copy our build
# COPY migrations /app/migrations
COPY --from=builder /app/target/release/$RUST_BIN /usr/local/bin/$RUST_BIN
ENTRYPOINT [ "/bin/bash", "-c", "$@"]
CMD ["${RUST_BIN}"]
