## Generate custom chain specification

The `chain-spec` file is essentially a genesis snapshot of your chain's state. It
defines the initial set of validators, the initial WASM binary of the runtime,
miscellaneous chain metadata, etc.

If you want to join an existing chain, you can skip this step as this only
needs to be performed once when bootstrapping the network. The only thing you
need from this part is the `chain-spec-raw.json` file that contains all
necessary information for your fresh node to join the existing network and
start syncing. You can download this file from
[here](https://github.com/agoraxyz/guild-network/releases/download/v0.0.0-alpha/chain-spec-raw.json)

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

