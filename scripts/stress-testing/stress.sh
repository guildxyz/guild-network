#!/bin/bash
# USAGE: bash ./stress.sh [number of threads] [tps] [total number of txs]
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
for i in $(seq 1 $1); do
    ../../target/release/gn-cli -i 65.108.102.250 stress --seed "$(head -c 10 /dev/urandom | sha1sum | head -c 10)" --tps $(($2 / $1)) -n $(($3 / $1)) register-other & 
done

wait $(jobs -p)
echo "All jobs done"
