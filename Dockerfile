FROM rustlang/rust:nightly-slim AS chef

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
    # && apt install build-essential git librocksdb-dev clang cmake llvm llvm-dev libssl-dev pkg-config -y
    && apt install build-essential librocksdb-dev libclang-dev clang cmake libssl-dev pkg-config -y

#RUN rustup toolchain install nightly \
RUN rustup target add wasm32-unknown-unknown --toolchain nightly

RUN cargo +nightly chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release 

FROM bitnami/minideb:bullseye AS runtime
RUN apt update -y \
    && apt upgrade -y \
    && apt install ca-certificates -y

RUN rm -rf /var/lib/apt/lists/* /var/lib/dpkg/* /var/cache/*

EXPOSE 30333 30333/udp 9944 9933

FROM runtime as gn-oracle
COPY --from=builder /opt/app/target/release/gn-oracle /usr/local/bin/

ARG ETHEREUM_RPC_TOKEN
ARG POLYGON_RPC_TOKEN
ARG GOERLI_RPC_TOKEN
ARG ARBITRUM_RPC_TOKEN
ARG PALM_RPC_TOKEN
ARG METIS_RPC_TOKEN
RUN echo -e 'ETHEREUM_RPC_TOKEN=$ETHEREUM_RPC_TOKEN\nPOLYGON_RPC_TOKEN=$POLYGON_RPC_TOKEN\nGOERLI_RPC_TOKEN=$GOERLI_RPC_TOKEN\nARBITRUM_RPC_TOKEN=$ARBITRUM_RPC_TOKEN\nPALM_RPC_TOKEN=$PALM_RPC_TOKEN\nMETIS_RPC_TOKEN=$METIS_RPC_TOKEN\n' > .env

CMD ["gn-oracle"]

FROM runtime AS gn-node
COPY --from=builder /opt/app/target/release/gn-node /usr/local/bin/
CMD ["gn-node"]
