#!/bin/sh

if [ $1 = "base" ]; then
	./target/release/node-template purge-chain \
		--base-path /tmp/$2 \
		--chain local -y
	
	./target/release/node-template \
		--base-path /tmp/$2 \
		--chain local \
		--alice \
		--port 30333 \
		--ws-port 9945 \
		--rpc-port 9933 \
		--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
		--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
		--validator
elif [ $1 = "child" ]; then
	./target/release/node-template purge-chain \
		--base-path /tmp/$2 \
		--chain local -y
	
	./target/release/node-template \
		--base-path /tmp/$2 \
		--chain local \
		--bob \
		--port 30334 \
		--ws-port 9946 \
		--rpc-port 9934 \
		--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
		--validator \
		--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
elif [ $1 = "dev" ]; then
	./target/release/node-template --dev
elif [ $1 = "clean" ]; then
	./target/release/node-template purge-chain --base-path /tmp/$2 -y
elif [ $1 = "benchmark" ]; then
	pallet=$2

	[ -z "$pallet" ] && pallet="template"

	./target/release/node-template benchmark pallet \
		--pallet "pallet-$pallet" \
		--extrinsic "*" \
		--execution=wasm \
		--wasm-execution=compiled \
		--steps 50 \
		--repeat 20 \
		--output ./pallets/"$pallet"/src/default_weights.rs
else
  echo "Invalid command"
fi
