#!/bin/sh

if [ $1 = "boot" ]; then
	./target/release/node-template purge-chain \
		--base-path /tmp/mynode \
		--chain local -y
	
	./target/release/node-template \
		--base-path /tmp/mynode \
		--chain chain-spec-raw.json \
		--alice \
		--port 30333 \
		--ws-port 9944 \
		--rpc-port 9933 \
		--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
		--rpc-external \
		--ws-external \
		--rpc-cors=all
elif [ $1 = "node" ]; then
	./target/release/node-template purge-chain \
		--base-path /tmp/mynode \
		--chain local -y
	
	./target/release/node-template \
		--base-path /tmp/mynode \
		--chain chain-spec-raw.json \
		--$2 \
		--port 30333 \
		--ws-port 9944 \
		--rpc-port 9933 \
		--bootnodes /ip4/$3/tcp/30333/p2p/$4 \
		--rpc-external \
		--ws-external \
		--rpc-cors=all
elif [ $1 = "build-spec" ]; then
	./target/release/node-template build-spec --disable-default-bootnode > chain-spec.json
	./target/release/node-template build-spec --chain=chain-spec.json --raw --disable-default-bootnode > chain-spec-raw.json
elif [ $1 = "clean" ]; then
	./target/release/node-template purge-chain --base-path /tmp/$2 -y
elif [ $1 = "benchmark" ]; then
	pallet=$2

	[ -z "$pallet" ] && pallet="template"

	./target/release/node-template benchmark pallet \
		--pallet pallet-$pallet \
		--extrinsic "*" \
		--execution=wasm \
		--wasm-execution=compiled \
		--steps 50 \
		--repeat 20 \
		--template frame-weight-template.hbs \
		--output ./pallets/$pallet/src/weights.rs
else
  echo "Invalid command"
fi
