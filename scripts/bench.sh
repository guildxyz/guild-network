#!/bin/sh
kebab="pallet-${1}"
snake=${kebab//-/_}
./target/release/gn-node benchmark pallet \
	--chain dev \
	--pallet $snake \
	--extrinsic "*" \
	--execution=wasm \
	--wasm-execution=compiled \
	--steps 50 \
	--repeat 20 \
	--output ./gn-pallets/$kebab/src/weights.rs
