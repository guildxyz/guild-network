#!/bin/sh
pallet=$1
./target/release/gn-node benchmark pallet \
	--chain dev \
	--pallet pallet_$pallet \
	--extrinsic "*" \
	--execution=wasm \
	--wasm-execution=compiled \
	--steps 50 \
	--repeat 20 \
	--output ./gn-pallets/pallet-$pallet/src/weights.rs
