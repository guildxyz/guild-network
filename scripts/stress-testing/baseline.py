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

BASELINE_TIMEFRAME = 250
END_CAPTURE_TIMEFRAME = 5
BLOCK_OVERHEAD = 187  # bytes

baseline_deviation = 0
capture = False
end_capture = END_CAPTURE_TIMEFRAME
z_size = 0
z_lat = 0
gather_baseline = True
last_timestamp = np.nan

print(
    f"Establishing baseline in {BASELINE_TIMEFRAME} blocks ({BASELINE_TIMEFRAME * 3} seconds)..."
)


def subscription_handler(obj, update_nr, subscription_id):
    global baseline_data, baseline_deviation, capture, end_capture, \
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

    if update_nr == BASELINE_TIMEFRAME - 1:
        return "Done"


# TEMP: see FIXME
try:
    result = substr.subscribe_block_headers(subscription_handler)
except KeyboardInterrupt:
    pass

# drop last n entries recorded after the capture should've ended
# # TEMP: see FIXME
print(capture_data)

capture_data['z-lat'] = (capture_data['latency'] - capture_data['latency'].mean()
                         ) / capture_data['latency'].std()
capture_data['z-size'] = (capture_data['latency'] - capture_data['latency'].mean()
                          ) / capture_data['latency'].std()

print(capture_data)

print(f"latency mean: {capture_data['latency'].mean():.3f}")
print(f"latency stdev: {capture_data['latency'].std():.3f}")

print(f"block size total: {capture_data['size'].sum()}")
print(f"block size mean: {capture_data['size'].mean():.3f}")
print(f"block size stdev: {capture_data['size'].std():.3f}")

print(f"extrinsic count: {capture_data['extrs'].sum()}")
print(f"extrinsic mean: {capture_data['extrs'].mean():.3f}")
print(f"extrinsic stdev: {capture_data['extrs'].std():.3f}")
print(
    f"extrinsics per second: {capture_data['extrs'].sum() / (capture_data['extrs'].count() * 3):.3f}"
)

print(f"Test lasted {len(capture_data.index)} blocks")
