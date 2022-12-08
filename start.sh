#!/bin/sh
if [ $1 = "boot" ]; then
	./target/release/gn-node purge-chain \
		--base-path /tmp/mynode \
		--chain chain-spec-raw.json -y
	
	./target/release/gn-node \
		--base-path /tmp/mynode \
		--chain chain-spec-raw.json \
		--alice \
		--port 30333 \
		--ws-port 9944 \
		--rpc-port 9933 \
		--node-key e845dfb08feee6de8d26200dfc1956873182ad91bcc8f35162e568716bb169cf \
		--rpc-external \
		--ws-external \
		--rpc-cors=all
elif [ $1 = "node" ]; then
	./target/release/gn-node purge-chain \
		--base-path /tmp/mynode \
		--chain chain-spec-raw.json -y
	
	./target/release/gn-node \
		--base-path /tmp/mynode \
		--chain chain-spec-raw.json \
		--$2 \
		--port 30333 \
		--ws-port 9944 \
		--rpc-port 9933 \
		--bootnodes /ip4/$3/tcp/30333/p2p/12D3KooWErJ9ChGGenCAmRQiiqmVxkZvsqkSB5GYVBGpN2rdfccE \
		--rpc-external \
		--ws-external \
		--rpc-cors=all
elif [ $1 = "api" ]; then
	subxt metadata -f bytes > gn-client/artifacts/metadata.scale
elif [ $1 = "dev" ]; then
	./target/release/gn-node --dev
elif [ $1 = "build-spec" ]; then
	./target/release/gn-node build-spec --disable-default-bootnode > chain-spec.json
	./target/release/gn-node build-spec --chain=chain-spec.json --raw --disable-default-bootnode > chain-spec-raw.json
elif [ $1 = "clean" ]; then
	./target/release/gn-node purge-chain --base-path /tmp/$2 -y
elif [ $1 = "benchmark" ]; then
	pallet=$2

	./target/release/gn-node benchmark pallet \
		--chain dev \
		--pallet pallet_$pallet \
		--extrinsic "*" \
		--execution=wasm \
		--wasm-execution=compiled \
		--steps 50 \
		--repeat 20 \
		--template frame-weight-template.hbs \
		--output ./gn-pallets/pallet-$pallet/src/weights.rs
else
  echo "Invalid command"
fi
