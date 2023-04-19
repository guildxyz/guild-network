# NOTE: this script requires the packages numpy, pandas and substrate-interface
# it is recommended to run this in a python venv and install the dependencies with the following commands:
# python3 -m venv venv
# . venv/bin/activate
# pip install -r requirements.txt

import time
import numpy as np
import pandas as pd
from substrateinterface import SubstrateInterface

substr = SubstrateInterface(url="wss://1.oracle.network.guild.xyz")
print("Connected")

capture_data = pd.DataFrame(columns=['timestamp', 'size', 'extrs', 'latency'])

BLOCK_OVERHEAD = 187  # bytes

capture = False
z_size = 0
z_lat = 0
gather_baseline = True
last_timestamp = np.nan

baseline = {
    "size_mean": 255.776,
    "size_stdev": 119.025,
    "lat_mean": 3.0,
    "lat_stdev": 0.001,
    "extr_mean": 1.316,
    "extr_stdev": 0.659
}
print(
    f"Baseline stats: {baseline}"
)


def subscription_handler(obj, update_nr, subscription_id):
    global capture, end_capture, \
        z_size, z_lat, gather_baseline, last_timestamp

    block_num = obj['header']['number']
    print(f"New block #{block_num}")

    block = substr.get_block(block_number=block_num)
    if block is None:
        return

    timestamp = block['extrinsics'][0].value['call']['call_args'][0]['value']
    block_size = sum([len(ext.data)
                     for ext in block['extrinsics']]) + BLOCK_OVERHEAD
    extr_num = len(block['extrinsics'])
    latency = (timestamp - last_timestamp) / 1000

    z_size = (block_size - baseline["size_mean"]) / baseline["size_stdev"]
    z_lat = (latency - baseline["lat_mean"]) / baseline["lat_stdev"]

    if abs(z_size) > 5 and not capture:
        capture = True
        print("CAPTURE TRIGGERED")
        print(f"z-size: {z_size}")
        print("Press Ctrl-C to end capture and show the results")

    if abs(z_lat) > 3:
        print("ANOMALY IN BLOCKTIME DETECTED")
        print(f"z-lat: {z_lat:.3f}")

    if capture:
        # FIXME: some tests take an unreasonably long "pause" in the middle
        # (going below baseline for longer than 6-8 blocks);
        # disabling automatic detection for now
        #
        # if abs(z_extr) < 3:
        #     end_capture -= 1
        # else:
        #     end_capture = END_CAPTURE_TIMEFRAME
        # if end_capture == 0:
        #     print("END OF CAPTURE")
        #     return "Done"
        capture_data.loc[block_num] = {  # type: ignore
            'timestamp': timestamp,
            'size': block_size,
            'extrs': extr_num,
            'latency': latency
        }
    print(f"Latency: {latency:.3f}")
    print(f"Block size: {block_size}")
    print(f"Number of extrinsics: {extr_num}")
    last_timestamp = timestamp


def compute_stats():
    # drop last n entries recorded after the capture should've ended
    # # TEMP: see FIXME
    capture_data['z-size'] = (capture_data['size'] - baseline["size_mean"]
                              ) / baseline["size_stdev"]
    print(capture_data)
    inp = input(
        "Enter how many rows should be removed from the end (after the end of the test): "
    )
    if inp == "":
        inp = "0"
    n = int(inp)
    capture_data.drop(capture_data.tail(n).index,
                      inplace=True)

    capture_data['z-lat'] = (capture_data['latency'] - capture_data['latency'].mean()
                             ) / capture_data['latency'].std()
    capture_data['z-size'] = (capture_data['size'] - capture_data['size'].mean()
                              ) / capture_data['size'].std()


def print_stats():
    print(capture_data)
    print("### Block time")
    if capture_data['latency'].std() > baseline["lat_stdev"] * 3:
        print(f"  - stdev: {capture_data['latency'].std():.3f}")
        print(f"  - mean: {capture_data['latency'].mean():.3f}")
        print(
            f"  - change in baseline: {capture_data['latency'].mean()  - baseline['lat_mean']:.3f}"
        )
    else:
        print("  - change in baseline: none")

    print("### Block size")
    print(f"  - total stored: {capture_data['size'].sum()}")
    print(f"  - stdev: {capture_data['size'].std():.3f}")
    print(f"  - mean: {capture_data['size'].mean():.3f}")
    print(
        f"  - change in baseline: {capture_data['size'].mean()  - baseline['size_mean']:.3f}"
    )

    print("### Extrinsics")
    print(f"  - total executed: {capture_data['extrs'].sum()}")
    print(f"  - extrinsics per second: {capture_data['extrs'].mean() * 3:.3f}")
    print(f"  - stdev: {capture_data['extrs'].std():.3f}")
    print(f"  - mean: {capture_data['extrs'].mean():.3f}")
    print(
        f"  - change in baseline: {capture_data['extrs'].mean()  - baseline['extr_mean']:.3f}"
    )
    print(f"Test lasted {len(capture_data.index)} blocks")


def main():
    # TEMP: see FIXME
    try:
        result = substr.subscribe_block_headers(subscription_handler)
    except KeyboardInterrupt:
        pass
    compute_stats()
    print_stats()


if __name__ == "__main__":
    main()
