#---
FROM rust:1.63.0-slim-buster as planner
WORKDIR app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

#---
FROM rust:1.63.0-slim-buster as cacher
WORKDIR app

RUN apt update -y \
    && apt upgrade -y \
    && apt install build-essential git librocksdb-dev clang cmake llvm llvm-dev libssl-dev pkg-config -y

RUN rustup toolchain install nightly \
    && rustup override set nightly \
    && rustup target add wasm32-unknown-unknown --toolchain nightly \
    && rustup component add clippy --toolchain nightly

RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo +nightly chef cook --release --recipe-path recipe.json

#---
FROM rust:1.63.0-slim-buster as build

RUN apt update -y \
    && apt upgrade -y \
    && apt install build-essential git librocksdb-dev clang cmake llvm llvm-dev libssl-dev pkg-config -y

RUN rustup toolchain install nightly \
    && rustup override set nightly \
    && rustup target add wasm32-unknown-unknown --toolchain nightly \
    && rustup component add clippy --toolchain nightly

WORKDIR /opt/app

COPY . /opt/app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release

#---
FROM alpine:3.16.2

RUN apk add --no-cache ca-certificates
COPY --from=build /opt/app/target/release/gn-node /usr/local/bin/gn-node

EXPOSE 30333 30333/udp 9944 9933 
ENTRYPOINT ["gn-node"]
