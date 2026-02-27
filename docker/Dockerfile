# Build stage
FROM rust:alpine as builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /build
COPY . .

# Build the simian-bin-monkey binary
RUN cargo build --release --bin simian-bin-monkey

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates curl bash openssl libgcc

# Create app user/group
RUN addgroup -g 10001 appuser && \
    adduser -D -u 10001 -G appuser appuser

# Create directories
RUN install -d -o 10001 -g 10001 /app && \
    install -d -o 10001 -g 10001 /app/logs

# Copy binary
COPY --from=builder --chown=10001:10001 /build/target/release/simian-bin-monkey /usr/local/bin/simian-bin-monkey

WORKDIR /app
USER 10001

ENV RUST_LOG=info
ENV SERVICE_NAME=simian-monkey

# Health check (basic process check)
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD ps aux | grep simian-bin-monkey || exit 1

ENTRYPOINT ["/usr/local/bin/simian-bin-monkey"]

