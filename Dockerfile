# 1. Use the Nightly compiler for Edition 2024 support
FROM rustlang/rust:nightly-bookworm as builder
WORKDIR /usr/src/aquathallyon
COPY . .
RUN cargo build --release

# 2. Run Phase
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
# Ensure the path matches your project name 'aquathallyon'
COPY --from=builder /usr/src/aquathallyon/target/release/aquathallyon /usr/local/bin/aquathallyon

CMD ["aquathallyon"]