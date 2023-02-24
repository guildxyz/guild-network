# Guild Network

Guild Network is (currently) an **experimental** Layer 1 blockchain that aims
to decentralize a subset of [Guild.xyz](https://guild.xyz)'s functionality. In
a nutshell, if you are unfamiliar with Guild.xyz, it provides a tool to build
and manage token-gated communities. Anybody can create their own guild and fill
it up with custom roles that users may obtain within the community. These roles
are usually tied to certain requirements that users need to meet in order to
get them.

Checking requirements is a fundamental building block that may need
verification data external to Guild Network. Like most blockchains out there,
Guild Network in itself is a closed system that cannot interact with the outer
world by itself. Therefore, Guild Network relies on a (currently) permissioned
oracle network that listen to on-chain events and fetch external data for
checking requirements. For now, the oracle network can only retrieve data
(token balances) from EVM blockchains.

The [repository](https://github.com/agoraxyz/guild-network), originally forked
from [Parity's `substrate-node-template`](https://github.com/substrate-developer-hub/substrate-node-template),
consists of crates that implement the described functionality above. Here's a
list with a brief overview for each crate:
- a node needs the following to join the network
	- `gn-node` - a full-fledged, Substrate-based blockchain node that can enter the network and participate in the decentralization of its functionality
	- `gn-runtime` - a modular, updatable, WASM-compatible [runtime](https://docs.substrate.io/fundamentals/runtime-development/) that describes the blockchain state and how it is modified via submitted transactions
	- `gn-pallets` - [pallets](https://docs.substrate.io/tutorials/work-with-pallets/) are essentially pluggable extensions for the runtime that customize how the runtime behaves
		- `pallet-guild` - this is the pallet through which users can submit their Guild-related transactions
		- `pallet-oracle` - this is the pallet through which oracle operators can interact with the network
- `gn-oracle` - a binary crate that oracle nodes need to run in order to subscribe to blockchain events, retrieve data from EVM blockchains and check requirements
- `gn-client` - essentially a wrapper around [a subxt client](https://docs.rs/subxt/latest/subxt/) that connects to network nodes for queries an submitting transactions
- `gn-wasm` - WASM wrappers around logic in `gn-client` used by the front-end application
- `gn-engine` - logic for creating requirements and verifying them
- `rusty-gate` - the alpha version external data retrieval used by the oracle nodes
- `gn-common` - common types, functions and wrappers that are used in most crates above
- `gn-test-data` - dummy test data for integration tests

The chain is currently in a free-to-use demo stage that **doesn't require** any
funds to interact with. However, you should always keep your private keys
secret and maintain healthy caution when trying the demo.

**NOTE** Guild Network is in alpha state and we are continuously working on
perfecting it. Expect bugs and even outages to the service while we strive
toward a decentralized solution that nobody can just switch off.

## Running a validator node

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

### Build and run a test node locally

Running a validator requires you to generate a couple of cryptographic keys for
which you need to build the source code first.

#### Build the source code

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

#### Run a single test-node

In case you want to quickly check your node, just run

```bash
./start.sh dev
```

This will spin up a clean node that you can interact with from the browser (see
last paragraph).

### Generate cryptographic keys

Every validator node will need to generate two cryptographic keys for `aura`
(block creation) and `grandpa` (block finalization)
[consensus](https://docs.substrate.io/fundamentals/consensus/).

#### Sr25519 for `aura`

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

#### Ed25519 for `grandpa`

Using the  secret phase from the Sr25519 key generation output run the
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

### Generate custom chain specification

If you want to join an existing chain, you can skip this step as this only
needs to be performed once when bootstrapping the network. The only thing you
need from this part is the `chain-spec-raw.json` file that contains all
necessary information for your fresh node to join the existing network and
start syncing. You can download this file from [here](https://todo.com).

The initial bootnode needs to generate a `chain-spec.json` file that contains
genesis configuration for the blockchain by running

```sh
./target/release/node-template build-spec --disable-default-bootnode > chain-spec.json
```

```json
{
  "name": "Local Testnet",
  "id": "local_testnet",
  "chainType": "Local",
  "bootNodes": [],
  "telemetryEndpoints": null,
  "protocolId": null,
  "properties": null,
  "consensusEngine": null,
  "codeSubstitutes": {},
  ..
}
```

Here, the `chainType` can be set to `Local`, `Development`, or `Live`. The
difference between these is that when the type is `Local` or `Development`,
the chain starts with pre-funded accounts that can interact with the network.
The `Live` type doesn't provide pre-funded accounts by default, you need to set
it manually. You may set the `name` and `id` fields if you want but note that
the `id` field determines where the chain data will be located on on your
computer. For example if `id = hello`, then the node database, keystore, and
other network-related stuff will be located in `/tmp/mynode/hello`, unless
specified otherwise when starting a live node. Therefore, make sure that the
`aura` and `grandpa` keys are saved under `/tmp/mynode/hello/keystore`,
otherwise you won't be able to validate and produce blocks. Generally there's
only two fields that definitely require modification.

The `aura` field needs to contain all `SS58` addresses of the Sr25519 keys
generated in the previous steps:

```json
"aura": { 
  "authorities": [
    "5CfBuoHDvZ4fd8jkLQicNL8tgjnK8pVG9AiuJrsNrRAx6CNW",
    "5CXGP4oPXC1Je3zf5wEDkYeAqGcGXyKWSRX2Jm14GdME5Xc5"
  ]
}
```

The `grandpa` field needs to contain all `SS58` addresses of the Ed25519 keys
generated in the previous steps:

```json
"grandpa": {
  "authorities": [
    [
      "5CuqCGfwqhjGzSqz5mnq36tMe651mU9Ji8xQ4JRuUTvPcjVN",
      1
    ],
    [
      "5DpdMN4bVTMy67TfMMtinQTcUmLhZBWoWarHvEYPM4jYziqm",
      1
    ]
  ]
},
```

The second element after the address is the voting weight of the nodes, here
set to 1 for both members.

After the specs are finalized, the `chain-spec.json` file needs to be converted
to raw format by running

```bash
./target/release/node-template build-spec --chain=chain-spec.json --raw --disable-default-bootnode > chain-spec-raw.json
```

Finally, make sure that every node operator receives the same
`chain-spec-raw.json` file.

### Insert keys into the keystore

Every validator node needs to insert their `aura` and `grandpa` keys into their
keystore which requires the following steps.

#### Adding the Sr25519 (`aura`) key to the node's keystore

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

#### Adding the Ed25519 (`grandpa`) key to the node's keystore

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

### Start the network nodes

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

---

#### Troubleshooting

In case you don't see the local node identity line in your bootnode's logs, then
you can pass an extra argument to your bootnode startup command:

```bash
--node-key 0000000000000000000000000000000000000000000000000000000000000001
```

which should be added by all other nodes at the end of line `--bootnodes
.../p2p/...` in their startup command options. The node key probably still
won't match but the logs will tell which is the correct bootnode identity and
adding that as an argument will solve the issue. This is kinda crappy and we
should definitely check how this really works.

---

If the nodes are successfully started, you should start seeing blocks being imported and finalized:

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


## Interacting with the network

In order to interact with the chain you need to have the [Metamask
wallet](https://metamask.io/) along with the
[polkadot.js](https://polkadot.js.org/extension/) extension installed. You can
find the front-end application to Guild Network [here](https://todo.com).

In the heart of Guild Network are guilds that can be created by anyone. Guilds
are initially empty but they can be filled up with various roles. Roles are
what make guilds  fill up with life and they usually come with a unique set of
requirements specified by the guild owner. Users who wish to attain a role in a
guild need to meet the respective requirements. Checking these requirements is
mostly the task of oracle operators who query external data sources to verify
whether a user has enough tokens on Etherum to join a guild and get a role
assigned.

Users first need to register identities (for example an Ethereum address) on
Guild Network that will be tied to their Substrate-address, i.e. the native
identity on Guild Network. Currently, users can register any address/public key
from a `secp256k1`, `ed25519` and/or `sr25519` elliptic curve signature scheme.
This is done via submitting a signature on-chain that is verified against the
registered address/public key. Users may also register Discord and Telegram
identities, but those are not verified yet (as they cannot really be verified
on-chain, we'll need oracles for that). 

**NOTE** all data on Guild Network is public, so only register identities that
you are comfortable with being aggregated and tied to your Substrate-address.

The registered (web3) identities will be used to check requirements and get
roles assigned to users. Other identities (e.g. Discord, Telegram) can be used
by third-party services (e.g. Discord and Telegram bots) that build on the
publicly available data aggregated on Guild Network so you can access gated
content for example.

### Substrate Front-end Template

The quickest and easiest way to interact with a network is via Parity's generic
[front-end
template](https://github.com/substrate-developer-hub/substrate-front-end-template)
for Substrate-based chains. You can submit extrinsics (transactions) and
monitor events via this tool from the browser.

Make sure you have `yarn` installed. Then you should clone and install the
front-end template by running

```bash
git clone https://github.com/substrate-developer-hub/substrate-front-end-template
cd substrate-front-end-template
yarn install
```

To connect to your node, you should modify the contents of
`src/config/development.json` such that the provider socket is set to

```json
{
  "PROVIDER_SOCKET": "ws://127.0.0.1:<port>"
}
```

where the port number is the number shown in the terminal output after you
started your node.

By running

```bash
yarn start
```

the app will open in your browser and you might see some pre-funded accounts from
which you can choose and interact with the blockchain via a the
`Pallet Interactor`.

### Polkadot.js

The [polkadot.js](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frpc-para.clover.finance#/explorer)
app lets you monitor a node which is incredibly useful if you are running a
validator. It will instantly try to connect to a node whose endpoint you need
to specify in the drop-down menu opening from the upper-left corner.

This is the main interface for submitting `sudo` transactions like registering
[new validators](https://github.com/gautamdhameja/substrate-validator-set/blob/master/docs/local-network-setup.md)
and oracle operators and updating the runtime of the network. Once Guild
Network leaves the testnet phase, the `sudo` account will be replaced by
decentralized governance.

## Pruning the chain

Whenever you want to delete your local copy of the chain, you can run

```bash
./target/release/node-template purge-chain --base-path /tmp/mynode --chain local_testnet
```

which will delete the database under `/tmp/mynode/local_testnet/db`. You can do
this manually as well.
