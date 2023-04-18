#!/bin/sh
# USAGE: sh ./stress.sh [number of threads] [tps per thread] [number of tx per thread]
for i in $(seq 1 $1); do
    ./target/release/gn-cli -i 65.108.102.250 stress --seed "$(head -c 10 /dev/urandom | sha1sum | head -c 10)" --tps $2 -n $3 register-other & 
done

wait $(jobs -p)
echo "All jobs done"