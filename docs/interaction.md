# Interacting with the network

In order to interact with the chain you need to have the [Metamask
wallet](https://metamask.io/) along with the
[polkadot.js](https://polkadot.js.org/extension/) extension installed. You can
find the front-end application to Guild Network [here](https://gn-rho.vercel.app/).

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

## Substrate Front-end Template

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

## Polkadot.js

The [polkadot.js](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frpc-para.clover.finance#/explorer)
app lets you monitor a node which is incredibly useful if you are running a
validator. It will instantly try to connect to a node whose endpoint you need
to specify in the drop-down menu opening from the upper-left corner.

This is the main interface for submitting `sudo` transactions like registering
[new validators](https://github.com/gautamdhameja/substrate-validator-set/blob/master/docs/local-network-setup.md)
and oracle operators and updating the runtime of the network. Once Guild
Network leaves the testnet phase, the `sudo` account will be replaced by
decentralized governance.
