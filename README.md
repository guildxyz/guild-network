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
oracle network that listens to on-chain events and fetch external data for
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
- `gn-api` - essentially a wrapper around [a subxt client](https://docs.rs/subxt/latest/subxt/) that connects to network nodes for queries an submitting transactions
- `gn-cli` - CLI for interacting with a network node or running an oracle node
- `gn-wasm` - WASM wrappers around logic in `gn-api` used by the front-end application
- `gn-engine` - logic for creating requirements and verifying them
- `gn-common` - common types, functions and wrappers that are used in most crates above
- `gn-test-data` - dummy test data for integration tests

The chain is currently in a free-to-use demo stage that **doesn't require** any
funds to interact with. However, you should always keep your private keys
secret and maintain healthy caution when trying the demo.

**NOTE** Guild Network is in alpha state and we are continuously working on
perfecting it. Expect bugs and even outages to the service while we strive
toward a decentralized solution that nobody can just switch off.

## Ongoing work
At the time of suspending active development, the state of the repo is as
follows:

Most recent changes can be found on branch `I137-oracle-refactor`. The goal of
this branch is to modify the oracle nodes such that it supports an identity
service along with checking requirements. The identity service should be
responsible to check whether non-crypto identities (e.g. Discord, Telegram)
belong to a given user.

Users can submit identity registration requests in `pallet-guild-identity`.
This pallet has been split from the original `pallet-guild` which is now
solely responsible for creating/managing and joining guilds.

The main part missing is the oracle refactor itself.

Related issues:

- [oracle refactor](https://github.com/agoraxyz/guild-network/issues/137)
- [identity service](https://github.com/agoraxyz/guild-network/issues/135)
- [requirement engine 2.0](https://github.com/agoraxyz/guild-rs/tree/oracle-integration)


## Documentation

- [running a validator](docs/validator.md)
- [interacting with the chain](docs/interaction.md)
- [genesis chain spec](docs/chain-spec.md)
- [runtime upgrades](docs/runtime-upgrades.md)
