FROM rustlang/rust:nightly-bullseye-slim AS chef

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
WORKDIR /opt/app
COPY --from=planner app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN apt update -y \
    && apt upgrade -y \
    && apt install build-essential librocksdb-dev libclang-dev clang cmake g++-multilib libssl-dev pkg-config protobuf-compiler -y

RUN rustup install nightly-2023-03-22 && rustup target add wasm32-unknown-unknown --toolchain nightly-2023-03-22

RUN cargo +nightly chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release 

FROM bitnami/minideb:bullseye AS runtime
RUN apt update -y \
    && apt upgrade -y \
    && apt install ca-certificates -y \ 
    && rm -rf /var/lib/apt/lists/* /var/lib/dpkg/* /var/cache/*

EXPOSE 30333 30333/udp 9944 9933

FROM runtime as gn-oracle
COPY --from=builder /opt/app/target/release/gn-oracle /usr/local/bin/

CMD ["gn-oracle"]

FROM runtime AS gn-node
COPY --from=builder /opt/app/target/release/gn-node /usr/local/bin/
CMD ["gn-node"]
