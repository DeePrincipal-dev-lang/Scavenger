# Build stage for Soroban contract
FROM rust:1.75 as builder

WORKDIR /build

RUN rustup target add wasm32-unknown-unknown

COPY stellar-contract/Cargo.toml stellar-contract/Cargo.lock* ./
COPY stellar-contract/src ./src

RUN cargo build --target wasm32-unknown-unknown --release 2>&1

# Install soroban-cli for optimization
RUN cargo install --locked soroban-cli --features opt

FROM scratch
COPY --from=builder /build/target/wasm32-unknown-unknown/release/*.wasm /
