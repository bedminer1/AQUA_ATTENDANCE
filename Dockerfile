FROM rust:1.75 as builder
WORKDIR /usr/src/aquatallyon
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/aquatallyon/target/release/aquatallyon /usr/local/bin/aquatallyon
CMD ["aquatallyon"]