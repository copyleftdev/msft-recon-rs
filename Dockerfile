FROM rust:1.72-slim AS builder

WORKDIR /usr/src/msft-recon-rs
COPY . .

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /opt/msft-recon-rs

COPY --from=builder /usr/src/msft-recon-rs/target/release/msft-recon-rs /opt/msft-recon-rs/
COPY --from=builder /usr/src/msft-recon-rs/config /opt/msft-recon-rs/config

# Create a non-root user to run the application
RUN useradd -ms /bin/bash recon
USER recon

ENTRYPOINT ["/opt/msft-recon-rs/msft-recon-rs"]
CMD ["--help"]
