### Build the source code

```bash
git clone git@github.com:agoraxyz/substrate-node-template.git
cd substrate-node-template
cargo build --release
```

**NOTE**: the build process will take somewhere between 20-30 minutes (depending
on the hardware) to build in `--release` mode.

### Run a single test-node

In case you want to quickly check your node, just run

```bash
./start.sh dev
```

This will spin up a clean node that you can interact with from the browser (see
last paragraph).

### Generate cryptographic keys

Every validator node will need to generate 2 cryptographic keys for `aura`
(block creation) and `grandpa` (block finalization).

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
following:

```bash
./target/release/node-template key inspect --password-interactive --scheme Ed25519 \
"pig giraffe ceiling enter weird liar orange decline behind total despair fly"
```

which, after providing the **same** password as above, will output something
like

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

Someone needs to generate a `chain-spec.json` file that contains specifications
for the blockchain by running

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
difference between these is that when the type is `Local` or `Development` ,
the chain starts with pre-funded accounts that can interact with the
network. The `Live` type doesn't provide pre-funded accounts. You
may set the `name` and `id` fields if you want but note that the
`id` field determines where the chain data will be located on on your
computer. For example if `id = hello`, then the node database,
keystore, and other network-related stuff will be located in
`/tmp/mynode/hello`. Therefore, make sure that the `aura` and
`grandpa` keys are saved under `/tmp/mynode/hello/keystore`,
otherwise you won't be able to validate and produce blocks.
Generally there's only two fields that definitely require
modification.

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

The second element after the address is the voting weight of the nodes, here set
to 1 for both members.

After the specs are finalized, the `chain-spec.json` file needs to be converted
to raw format by running

```bash
./target/release/node-template build-spec --chain=chain-spec.json --raw --disable-default-bootnode > chain-spec-raw.json
```

Finally, make sure that every node operator receives the same
`chain-spec-raw.json` file.

### Insert keys into the keystore

Everybody who wishes to participate in the network by running a node will need
to perform the following steps:

Add the Sr25519 key to the node's keystore:

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

Add the Ed25519 key to the node's keystore:

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

### Setup Tailscale

Since the nodes won't find each other if you just provide the IP address of the
bootnode, you need to set up `tailscale`. First, make sure to log in to the
`substrate.pista@gmail.com` Google account (ask Mark for the password). Then go
to the [tailscale](https://tailscale.com/) website and press Log In. You will be
prompted by an authentication window: you should log in with Google. Install the
`tailscale` cli app (the website will provide the link for it) and then you
should run

```bash
sudo tailscale up
```

Then, by running

```bash
tailscale status
```

you should see the network participants with their virtual IP addresses:

```text
100.x.x.x   turbineblade        substrate.pista@ linux   -
100.x.x.x  gyozosz-ms-7b98      substrate.pista@ linux   - 
```

You can test the connection by pinging one of the IP addresses in the network.
If you receive a response, you're all set for the final step.

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
servers.
By default they only accept `localhost` and `polkadot.js` origins so it should
be set to accept all origins.

Next, each computer that's part of the `tailscale` network should run something
like:

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
should be copied from the output of

```bash
tailscale status
```

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

If the nodes are successfully started, you should start seeing blocks being
finalized:

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

### Interacting with the network

Make sure you have `yarn` installed. Then you should clone and install the
frontend template by running

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

the app will open in your browser and you will see some pre-funded accounts from
which you can choose and interact with the blockchain via a the
`Pallet Interactor`.

### Pruning the chain

Whenever you want to delete your local copy of the chain, you can run

```bash
./target/release/node-template purge-chain --base-path /tmp/mynode --chain local_testnet
```

which will delete the database under `/tmp/mynode/local_testnet/db`. You can do
this manually as well.
