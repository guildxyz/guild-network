# Substrate oracle client

This is a CLI client which connects to the specified Substrate node and listens
for events. In our case these events are oracle requests and after decoding the
data from the event we fetch the results from the given API and send the results
back to the node inside a transaction to provide off-chain data for our pallets.

## Instructions

### Building the client

```bash
git clone git@github.com:agoraxyz/substrate-oracle-client.git
cd substrate-oracle-client
cargo build
```

### Running the client

**NOTE**: it is important to make sure you already started your Substrate node
so that the client can connect to it.

```bash
# if you have already compiled the source in the previous step run
./target/release/substrate-client

# otherwise
cargo run -- <parameters>

# tests
cargo t // note that test threads are limited to 1 in .cargo/config.toml

# flood tests
cargo t --features flood-tests

# bencharks
cargo bench --features benchmarking
```

#### Parameters

`--log` / `-l` - specify the log level (`debug` \ `info` \ `warn` \ `error`),
defaults to `warn` when not provided.

`--node-ip` / `-i` - specify Substrate node IP address, defaults to `127.0.0.1`.

`--node-port` / `-p` - specify Substrate node port number, defaults to `9944`.

`--id` - specify operator account (Alice, Bob, Charlie, Dave, Eve, Ferdie),
defaults to Alice.
