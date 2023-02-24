# Running a validator node

If you are up for a challenge and want to participate in the network by running
your own Guild Network node you need the following

- a unix-based machine with the Rust toolchain and `cargo` (Rust's package manager) installed (we haven't tried Windows builds yet)
```sh
# install rustup
curl https://sh.rustup.rs -sSf | sh
```
- some packages that might not be pre-installed on a fresh build (package names may differ depending on the installed OS)
	- `librocksdb-dev`
	- `libclang-dev`
	- `clang`, `cmake
	- `g++-multilib`
	- `libssl-dev`
	- `pkg-config`
	- `protobuf-compiler`
- for our nodes we use servers with the following setup
	- hardware - we use a setup [recommended for Polkadot validators](https://wiki.polkadot.network/docs/maintain-guides-how-to-validate-polkadot#reference-hardware)
	- costs - depends on the service you are using, but for our nodes currently
		- €70 /month/node
		- €50/node one-time setup fee

To make life easier, here's a checklist you need to go through to become a validator:

- [ ] clone the repository
- [ ] build the source code
- [ ] download the genesis chain specification
- [ ] generate cryptographic validator keys
  - [ ] Sr25519 for `aura`
  - [ ] Ed25519 for `grandpa`
- [ ] add the cryptographic validator keys to your node's local keystore
  - [ ] add the `aura` key
  - [ ] add the `grandpa` key
- [ ] start your validator node
- [ ] set your public session keys
  - [ ] make an RPC calls to rotate your keys
  - [ ] set the rotated keys as your session keys
- [ ] notify us so we can register you as a validator via the `sudo` account (this step will be replaced by governance on the mainnet)

## Build and run a test node locally

Running a validator requires you to generate a couple of cryptographic keys for
which you need to build the source code first.

### Build the source code

To build the source code you need to clone it first:

```bash
git clone git@github.com:agoraxyz/guild-network.git
cd guild-network
cargo build --release
```

**NOTE**: the build process will take somewhere between 20-30 minutes
(depending on the hardware) to build in `--release` mode. For optimal
performance, however, it is highly advised to build the code in `--release`
mode.

### Run a single test-node

In case you want to quickly check your node, just run

```bash
./start.sh dev
```

This will spin up a clean node that you can [interact with from the browser](https://github.com/agoraxyz/guild-network/docs/interaction.md). You should see it importing and finalizing blocks in the logs, something along the lines of:

```text
2022-07-14 12:04:12 🙌 Starting consensus session on top of parent 0xd4df501cbe450d3465cc7074ce2e3116b8e481e1d8bff347a0491785a31c118e    
2022-07-14 12:04:12 🎁 Prepared block for proposing at 49 (0 ms) [hash: 0x7198e07fe4e1eb07f49282712be07bc386dd1cc11813ee24ae4e532ca2ee83ef; parent_hash: 0xd4df…118e; extrinsics (1): [0x4dfa…d63c]]    
2022-07-14 12:04:12 🔖 Pre-sealed block for proposal at 49. Hash now 0xcc03e0613019a4ca703901aa1632640b39c16a3b3dec46c0aed4673bff2c186e, previously 0x7198e07fe4e1eb07f49282712be07bc386dd1cc11813ee24ae4e532ca2ee83ef.    
2022-07-14 12:04:12 ✨ Imported #49 (0xcc03…186e)    
2022-07-14 12:04:15 💤 Idle (1 peers), best: #49 (0xcc03…186e), finalized #47 (0xc4c9…b00f), ⬇ 0.6kiB/s ⬆ 0.7kiB/s    
2022-07-14 12:04:18 ✨ Imported #50 (0xb816…1eb4)    
2022-07-14 12:04:20 💤 Idle (1 peers), best: #50 (0xb816…1eb4), finalized #48 (0xd4df…118e), ⬇ 0.7kiB/s ⬆ 0.7kiB/s    
2022-07-14 12:04:24 🙌 Starting consensus session on top of parent 0xb816b1453573f4cac7d521a40fea3bdf3905a14c50a030898f90745fb7ce1eb4    
2022-07-14 12:04:24 🎁 Prepared block for proposing at 51 (0 ms) [hash: 0x69dd14bcee632604ef6657f02b942f8dbd9cc8f938f2dd0bad7c1629fe7b3095; parent_hash: 0xb816…1eb4; extrinsics (1): [0xd77e…5fc1]]    
2022-07-14 12:04:24 🔖 Pre-sealed block for proposal at 51. Hash now 0x295f81e99454b89abcfe397a6b9eaedf03d00b022bcbfadb2ef7fb8e42075f85, previously 0x69dd14bcee632604ef6657f02b942f8dbd9cc8f938f2dd0bad7c1629fe7b3095.    
2022-07-14 12:04:24 ✨ Imported #51 (0x295f…5f85)
```

## Generate cryptographic keys

Every validator node will need to generate two cryptographic keys for `aura`
(block creation) and `grandpa` (block finalization)
[consensus](https://docs.substrate.io/fundamentals/consensus/).

### Sr25519 for `aura`

```bash
./target/release/node-template key generate --scheme Sr25519 --password-interactive
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
./target/release/node-template key inspect --password-interactive --scheme Ed25519 \
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
./target/release/node-template key insert --base-path /tmp/mynode \
  --chain chain-spec-raw.json \
  --scheme Sr25519 \
  --suri <your-secret-seed> \
  --password-interactive \
  --key-type aura
```

**NOTE**: use the same secret seeds and password as during the key generation
step.

### Adding the Ed25519 (`grandpa`) key to the node's keystore

```bash
./target/release/node-template key insert \
  --base-path /tmp/mynode \
  --chain chain-spec-raw.json \
  --scheme Ed25519 \
  --suri <your-secret-seed> \
  --password-interactive \
  --key-type gran
```

**NOTE**: use the same secret seeds and password as during the key generation
step.

Finally, verify that the output of

```bash
ls /tmp/mynode/chains/testnet/keystore
```

resembles this:

```text
617572611441ddcb22724420b87ee295c6d47c5adff0ce598c87d3c749b776ba9a647f04
6772616e1441ddcb22724420b87ee295c6d47c5adff0ce598c87d3c749b776ba9a647f04
```

## Start the network node

First, start the bootnode by running

```bash
./target/release/node-template \
  --base-path /tmp/mynode \
  --chain ./chain-spec-raw.json \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --rpc-methods Unsafe \
  --ws-external \
  --rpc-cors=all \
  --name MyNode \
  --password-interactive
```

This should output a ton of lines but you should find this particular line:

```text
2021-11-03 15:32:15 🏷 Local node identity is: 12D3KooWLmrYDLoNTyTYtRdDyZLWDe1paxzxTw5RgjmHLfzW96SX
```

because you'll need the local node identity for the other nodes. Note the
`--ws-external` and the `--rpc-cors=all` flags. The former lets your node listen
to all websocket interfaces, not just the local ones. This is required for the
node to accept websocket subscriptions on deployed, non-local networks. The
latter flag specifies browser Origins allowed to access the HTTP and WS RPC
servers. By default they only accept `localhost` and `polkadot.js` origins so it should
be set to accept all origins.

Next, each validator needs to run

```bash
./target/release/node-template \
  --base-path /tmp/mynode \
  --chain ./chain-spec-raw.json \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --rpc-methods unsafe \
  --name MyNode \
  --ws-external \
  --rpc-cors=all \
  --bootnodes /ip4/100.x.x.x/tcp/30333/p2p/12D3KooWLmrYDLoNTyTYtRdDyZLWDe1paxzxTw5RgjmHLfzW96SX \
  --password-interactive
```

where the most important line is this:

```bash
  --bootnodes /ip4/100.x.x.x/tcp/30333/p2p/12D3KooWLmrYDLoNTyTYtRdDyZLWDe1paxzxTw5RgjmHLfzW96SX \
```

This line tells the node to look for the bootnode at address `100.x.x.x` which
should be copied from the output of TODO

Furthermore, the local node identity should be added after `p2p/...`.

## Set session keys

`aura` and `grandpa` consensus happens in sessions with each session holding a set of validators to particpate in the consensus. Therefore, after the node is up and running, you need to get your public `aura` and `grandpa` keys from the node. You need to perform [steps 4, 5 and 6](https://github.com/gautamdhameja/substrate-validator-set/blob/master/docs/local-network-setup.md#step-4).

In case you get an error in step 5, that is probably because the keys received in step 4 are actually 64 bytes instead of 32. In that case, split the key received in step 4 in half and input the first half in the `aura` and the second half in the `grandpa` field (each with a `0x` prefix).

After you've successfully submitted the transaction (you should get a green tick icon in the upper right corner) let us know so we can register your validator via the `sudo` pallet.
