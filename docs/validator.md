# Running a validator node

If you are up for a challenge and want to participate in the network by running
your own Guild Network node you need the following

- a linux-based machine running either
	- Debian (`bullseye` or `bookworm` version)
	- Ubuntu (at least `20.04-focal` version, our nodes run on `22.04-jammy`)
	- Arch Linux (we only checked with the `6.1.12` kernel version)
- you need to install the Rust toolchain and `cargo` (Rust's package manager)
```sh
# install rustup
curl https://sh.rustup.rs -sSf | sh
```
- you need to add the `wasm32-unknown-unknown` target on the `nightly` channel:
```sh
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```
- some packages that might not be pre-installed on a fresh build (package names may differ depending on the installed OS)
	- `librocksdb-dev`
	- `libclang-dev`
	- `clang`, `cmake`
	- `g++-multilib`
	- `libssl-dev`
	- `llvm`, `llvm-dev`
	- `pkg-config`
	- `protobuf-compiler`
- for our nodes we use servers with the following setup
	- hardware - we use a setup [recommended for Polkadot validators](https://wiki.polkadot.network/docs/maintain-guides-how-to-validate-polkadot#reference-hardware)
	- costs - depends on the service you are using, but for our nodes currently
		- €70/month/node
		- €50/node one-time setup fee

To make life easier, here's a checklist you need to go through to become a validator:

- [ ] clone the repository (you might need to [set up an SSH key](https://docs.github.com/en/authentication/connecting-to-github-with-ssh) first)
- [ ] build the source code
- [ ] download the genesis chain specification [`chain-spec-raw.json`](https://github.com/agoraxyz/guild-network/releases/download/v0.0.0-alpha/chain-spec-raw.json)
- [ ] generate cryptographic validator keys
  - [ ] Sr25519 for `aura`
  - [ ] Ed25519 for `grandpa`
- [ ] add the cryptographic validator keys to your node's local keystore
  - [ ] add the `aura` key
  - [ ] add the `grandpa` key
- [ ] start your validator node
- [ ] set your public session keys
  - [ ] make an RPC call to rotate your keys
  - [ ] notify us so we can register you as a validator via the `sudo` account (this step will be replaced by governance on the mainnet)

## Build and run a test node locally

Running a validator requires you to generate a couple of cryptographic keys for
which you need to build the source code first.

### Build the source code

To build the source code you need to clone it first:

```bash
# https
# git clone https://github.com/agoraxyz/guild-network.git
# ssh
git clone git@github.com:agoraxyz/guild-network.git
cd guild-network
cargo build --release
```

**NOTE**: the build process will take somewhere between 20-30 minutes
(depending on the hardware) to build in `--release` mode. For optimal
performance, however, it is highly advised to build the code in `--release`
mode.

**TROUBLESHOOTING**: if you get `secp256k1`-related warnings/errors like `No available targets are compatible with this triple` and the code fails to build you might need to run `export CC=gcc` before `cargo build --release`.

### Run a single test-node

In case you want to quickly check your node, run the following from the workspace root

```bash
./scripts/dev.sh
```

This will spin up a clean node that you can [interact with from the browser](https://github.com/agoraxyz/guild-network/docs/interaction.md). You should see it importing and finalizing blocks in the logs, something along the lines of:

```text
2023-03-06 10:13:11 Substrate Node    
2023-03-06 10:13:11 ✌️  version 0.0.0-alpha-2d65307203d    
2023-03-06 10:13:11 ❤️  by , 2017-2023    
2023-03-06 10:13:11 📋 Chain specification: Development    
2023-03-06 10:13:11 🏷  Node name: common-flag-2215    
2023-03-06 10:13:11 👤 Role: AUTHORITY    
2023-03-06 10:13:11 💾 Database: RocksDb at /tmp/substrateJDibik/chains/dev/db/full    
2023-03-06 10:13:11 ⛓  Native runtime: guild-network-101 (guild-network-1.tx1.au1)    
2023-03-06 10:13:11 🔨 Initializing Genesis block/state (state: 0xee74…0185, header-hash: 0x9fc1…2e54)    
2023-03-06 10:13:11 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.    
2023-03-06 10:13:11 Using default protocol ID "sup" because none is configured in the chain specs    
2023-03-06 10:13:11 🏷  Local node identity is: 12D3KooWJVrSt1ukXdmM94Tu2RSCkRnzumYtmFJuWRZcwVXCuHti    
2023-03-06 10:13:11 💻 Operating system: linux    
2023-03-06 10:13:11 💻 CPU architecture: x86_64    
2023-03-06 10:13:11 💻 Target environment: gnu    
2023-03-06 10:13:11 💻 CPU: AMD Ryzen 5 3600 6-Core Processor    
2023-03-06 10:13:11 💻 CPU cores: 6    
2023-03-06 10:13:11 💻 Memory: 7872MB    
2023-03-06 10:13:11 💻 Kernel: 6.1.12-arch1-1    
2023-03-06 10:13:11 💻 Linux distribution: Arch Linux    
2023-03-06 10:13:11 💻 Virtual machine: no    
2023-03-06 10:13:11 📦 Highest known block at #0    
2023-03-06 10:13:11 〽️ Prometheus exporter started at 127.0.0.1:9615    
2023-03-06 10:13:11 Running JSON-RPC HTTP server: addr=127.0.0.1:9933, allowed origins=["*"]    
2023-03-06 10:13:11 Running JSON-RPC WS server: addr=127.0.0.1:9944, allowed origins=["*"]    
2023-03-06 10:13:12 🙌 Starting consensus session on top of parent 0x9fc14b5ce5543f8ce21e87a91b094df4ea2e03f1960fcf39e1cf49ffbfa72e54    
2023-03-06 10:13:12 🎁 Prepared block for proposing at 1 (0 ms) [hash: 0x178ec36ac2a9e9f7613816cd1ba8f978a6052471f8ee703a6e75a419e287e446; parent_hash: 0x9fc1…2e54; extrinsics (1): [0x286a…46d6]]    
2023-03-06 10:13:12 🔖 Pre-sealed block for proposal at 1. Hash now 0xbb88abc6ffaa246af0d67a90688cbd26b02eca8e0407d61e76e05e5621af9c0c, previously 0x178ec36ac2a9e9f7613816cd1ba8f978a6052471f8ee703a6e75a419e287e446.    
2023-03-06 10:13:12 ✨ Imported #1 (0xbb88…9c0c)    
2023-03-06 10:13:16 💤 Idle (0 peers), best: #1 (0xbb88…9c0c), finalized #0 (0x9fc1…2e54), ⬇ 0 ⬆ 0
```

**NOTE**: This command does not deploy your node, it only starts a local development node to ensure the source was built properly and the node behaves as it should.

## Generate cryptographic keys

Every validator node will need to generate two cryptographic keys for `aura`
(block creation) and `grandpa` (block finalization)
[consensus](https://docs.substrate.io/fundamentals/consensus/).

### Sr25519 for `aura`

```bash
./target/release/gn-node key generate --scheme Sr25519 --password-interactive
```

This will prompt the user to provide a password and will output something like

```text
Secret phrase:  pig giraffe ceiling enter weird liar orange decline behind total despair fly
Secret seed:       0x0087016ebbdcf03d1b7b2ad9a958e14a43f2351cd42f2f0a973771b90fb0112f
Public key (hex):  0x1a4cc824f6585859851f818e71ac63cf6fdc81018189809814677b2a4699cf45
Account ID:        0x1a4cc824f6585859851f818e71ac63cf6fdc81018189809814677b2a4699cf45
Public key (SS58): 5CfBuoHDvZ4fd8jkLQicNL8tgjnK8pVG9AiuJrsNrRAx6CNW
SS58 Address:      5CfBuoHDvZ4fd8jkLQicNL8tgjnK8pVG9AiuJrsNrRAx6CNW
```

Here the `SS58` address is the encoded public key which will be needed at later
steps.

### Ed25519 for `grandpa`

Using the secret phase from the Sr25519 key generation output run the
following with the **same** password as before

```bash
./target/release/gn-node key inspect --password-interactive --scheme Ed25519 \
"pig giraffe ceiling enter weird liar orange decline behind total despair fly"
```

which will output something like

```text
Secret phrase `pig giraffe ceiling enter weird liar orange decline behind total despair fly` is account:
Secret seed:       0x0087016ebbdcf03d1b7b2ad9a958e14a43f2351cd42f2f0a973771b90fb0112f
Public key (hex):  0x2577ba03f47cdbea161851d737e41200e471cd7a31a5c88242a527837efc1e7b
Public key (SS58): 5CuqCGfwqhjGzSqz5mnq36tMe651mU9Ji8xQ4JRuUTvPcjVN
Account ID:        0x2577ba03f47cdbea161851d737e41200e471cd7a31a5c88242a527837efc1e7b
SS58 Address:      5CuqCGfwqhjGzSqz5mnq36tMe651mU9Ji8xQ4JRuUTvPcjVN
```

where the `SS58` address will be needed for later steps.

## Insert keys into the keystore

Every validator node needs to insert their `aura` and `grandpa` keys into their
keystore which requires the following steps.

### Adding the Sr25519 (`aura`) key to the node's keystore

```bash
./target/release/gn-node key insert \
  --base-path [data-dir] \
  --chain chain-spec-raw.json \
  --scheme Sr25519 \
  --suri [your-secret-seed] \
  --password-interactive \
  --key-type aura
```

**NOTE**: use the same secret seeds and password as during the key generation
step.

### Adding the Ed25519 (`grandpa`) key to the node's keystore

```bash
./target/release/gn-node key insert \
  --base-path [data-dir] \
  --chain chain-spec-raw.json \
  --scheme Ed25519 \
  --suri [your-secret-seed] \
  --password-interactive \
  --key-type gran
```

**NOTE**: use the same secret seeds and password as during the key generation
step.

Finally, verify that the output of

```bash
ls [data-dir]/chains/[chain name]/keystore # e.g. /tmp/mynode/chains/testnet/keystore
```

resembles this:

```text
617572611441ddcb22724420b87ee295c6d47c5adff0ce598c87d3c749b776ba9a647f04
6772616e1441ddcb22724420b87ee295c6d47c5adff0ce598c87d3c749b776ba9a647f04
```

## Running an (unsafe) validator node

In order to register as a validator you will need to call the
`author_rotateKey` RPC method at one point which is an unsafe RPC call. Thus,
when you first start your node you need to enable unsafe RPC calls until
you've successfully joined the validator set. [A Substrate
  seminar](https://github.com/substrate-developer-hub/substrate-seminar/blob/main/scheduled/2022/03-15-testnet-validators.md)
also suggests starting a node like this.

```bash
./target/release/gn-node \
        --base-path [data dir] \
        --chain [raw-chain-spec] \
        --validator \
        --name [name] \
        --bootnodes [bootnode multiaddr] \
        --enable-offchain-indexing true \
        --unsafe-ws-external \
        --unsafe-rpc-external \
        --rpc-methods=Unsafe \
        --rpc-cors=all \
        --ws-max-connections 5000 \
        --pruning=archive
```

You should download the raw chain specification from
[here](https://github.com/agoraxyz/guild-network/releases/download/v0.0.0-alpha/chain-spec-raw.json)
and plug that into `[raw-chain-spec]`. `[data-dir]` and `[name]` are free to
choose. However, line `--bootnodes [bootnode multiaddress]` should look like
this

```bash
  --bootnodes /ip4/65.108.102.250/tcp/30333/p2p/12D3KooWErJ9ChGGenCAmRQiiqmVxkZvsqkSB5GYVBGpN2rdfccE
```

## Set session keys

For this step you'll need to connect to our bootnode which provides a secure
websocket connection that allows the polkadot.js app to connect. Check out the
[link](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F1.oracle.network.guild.xyz#/explorer).

Make sure you have installed the [polkadot.js wallet
extension](https://polkadot.js.org/extension/). If you already have a Polkadot
address in your wallet, you don't need to generate a fresh keypair, otherwise
make sure you generate a new one and save its mnemonic seed. Reach out to us
for some testnet tokens before the next steps.

`aura` and `grandpa` consensus happens in sessions with each session holding a
set of validators to particpate in the consensus. Therefore, after the node is
up and running, you need to get your public `aura` and `grandpa` keys from the
node. You need to perform
[steps 4 to 6](https://github.com/gautamdhameja/substrate-validator-set/blob/master/docs/local-network-setup.md#step-4).

**NOTE**: You need to call `rotate_keys

In case you get an error in step 5, that is probably because the keys received
in step 4 are actually 64 bytes instead of 32. In that case, split the key
received in step 4 in half and input the first half in the `aura` and the
second half in the `grandpa` field (each with a `0x` prefix).

After you've successfully submitted the transaction (you should get a green
tick icon in the upper right corner) let us know so we can register your
validator via the `sudo` pallet.

## Running a (safe) validator node

After you've successfully joined as a validator, you may choose to restart your
node via one of the following options. Note, that if you haven't started your
node with `--pruning=archive` before, then you won't be able to start an
archive node unless you prune the node database and start syncing again.

- archive node (recommended, because it keeps the whole chain state in the database - for reference,
  a Polkadot archive node has a [~560GB state as of nov. 2022](https://paranodes.io/DBSize))

```bash
./target/release/gn-node \
        --base-path [data dir] \
        --chain [raw-chain-spec] \
        --validator \
        --name [name] \
        --bootnodes [bootnode multiaddr] \
        --enable-offchain-indexing true \
        --pruning=archive
```

- rpc node (keeps rpc ports open for safe rpc methods)

```bash
./target/release/gn-node \
        --base-path [data dir] \
        --chain [raw-chain-spec] \
        --validator \
        --name [name] \
        --bootnodes [bootnode multiaddr] \
        --enable-offchain-indexing true \
        --unsafe-ws-external \
        --rpc-methods Safe \
        --rpc-cors=all \
        --ws-max-connections 5000 \
        --pruning=archive
```

- pruning validator node (prunes old blocks from the database while only keeping
  the most recent 256)

```bash
./target/release/gn-node \
        --base-path [data dir] \
        --chain [raw-chain-spec] \
        --validator \
        --name [name] \
        --bootnodes [bootnode multiaddr] \
        --enable-offchain-indexing true \
        --pruning=256
```

**NOTE**: None of the above nodes will expose unsafe RPC methods.

## Set up secure websocket service on your server

If you want the chain explorer and the frontend to be able to connect to your
node via a secure websocket connection (`wss`) you need to set up your server
accordingly. We use [caddy](https://caddyserver.com/) on our bootnode's server
to enable secure connections.

If you have experience with such setups than go ahead and do it, otherwise,
details coming soon...
