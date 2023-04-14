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

baseline_data = pd.DataFrame(columns=['size', 'extrs', 'latency'])
capture_data = pd.DataFrame(columns=['timestamp', 'size', 'extrs', 'latency'])

BASELINE_TIMEFRAME = 25
END_CAPTURE_TIMEFRAME = 5

baseline_deviation = 0
capture = False
end_capture = END_CAPTURE_TIMEFRAME
z_extr = 0
z_lat = 0
gather_baseline = True
last_timestamp = np.nan

print(
    f"Establishing baseline in {BASELINE_TIMEFRAME} blocks ({BASELINE_TIMEFRAME * 3} seconds)..."
)


def subscription_handler(obj, update_nr, subscription_id):
    global baseline_data, baseline_deviation, capture, end_capture, \
        z_extr, z_lat, gather_baseline, last_timestamp

    block_num = obj['header']['number']
    print(f"New block #{block_num}")

    block = substr.get_block(block_number=block_num)
    if block is None:
        return

    timestamp = block['extrinsics'][0].value['call']['call_args'][0]['value']
    latency = (timestamp - last_timestamp) / 1000
    extr_num = len(block['extrinsics'])

    if gather_baseline:
        baseline_data.loc[block_num] = {
            'size': 0,
            'extrs': extr_num,
            'latency': latency
        }
        if update_nr == BASELINE_TIMEFRAME - 1:
            gather_baseline = False
            print(
                f"Established baseline with {len(baseline_data)} blocks.",
            )
            print(
                f"baseline stdev (extrinsics): {baseline_data['extrs'].std():.3f}"
            )
            print(
                f"baseline stdev (latency): {baseline_data['latency'].std():.3f}"
            )
    else:
        z_extr = (
            extr_num - baseline_data['extrs'].mean()) / baseline_data['extrs'].std()
        z_lat = (
            latency - baseline_data['latency'].mean()) / baseline_data['latency'].std()

    if abs(z_extr) > 3 and not capture:
        capture = True
        print("CAPTURE TRIGGERED")
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
            'size': 0,
            'extrs': extr_num,
            'latency': latency
        }
    print(f"Latency: {latency:.3f}")
    print(f"Number of extrinsics: {extr_num}")
    last_timestamp = timestamp


# TEMP: see FIXME
try:
    result = substr.subscribe_block_headers(subscription_handler)
except KeyboardInterrupt:
    pass

# drop last n entries recorded after the capture should've ended
# # TEMP: see FIXME
print(capture_data)
n = int(input("Enter how many rows should be removed from the end (after the end of the test): "))
capture_data.drop(capture_data.tail(END_CAPTURE_TIMEFRAME - 1).index,
                  inplace=True)
capture_data['z-lat'] = (capture_data['latency'] - capture_data['latency'].mean()
                         ) / capture_data['latency'].std()

print(capture_data)

print(f"latency mean: {capture_data['latency'].mean():.3f}")
print(f"latency stdev: {capture_data['latency'].std():.3f}")

print(f"extrinsic count: {capture_data['extrs'].sum()}")
print(f"extrinsic mean: {capture_data['extrs'].mean():.3f}")
print(f"extrinsic stdev: {capture_data['extrs'].std():.3f}")
print(
    f"extrinsics per second: {capture_data['extrs'].sum() / (capture_data['extrs'].count() * 3):.3f}"
)

print(f"Test lasted {len(capture_data.index)} blocks")

print("blocksize: TODO")
