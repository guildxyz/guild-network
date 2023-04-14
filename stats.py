import time
import numpy as np
import pandas as pd
from substrateinterface import SubstrateInterface
from pprint import pprint

substr = SubstrateInterface(url="wss://1.oracle.network.guild.xyz")
print("Connected")

baseline_data = pd.DataFrame(columns=['size', 'extrs', 'latency'])
capture_data = pd.DataFrame(columns=['timestamp', 'size', 'extrs', 'latency'])

BASELINE_TIMEFRAME = 10
END_CAPTURE_TIMEFRAME = 5

baseline_deviation = 0
capture = False
end_capture = END_CAPTURE_TIMEFRAME
z_score = 0
gather_baseline = True
last_timestamp = np.nan

print(
    f"Establishing baseline in {BASELINE_TIMEFRAME} blocks ({BASELINE_TIMEFRAME * 3} seconds)..."
)


def subscription_handler(obj, update_nr, subscription_id):
    global baseline_data, baseline_deviation, capture, end_capture, z_score, gather_baseline, last_timestamp

    block_num = obj['header']['number']
    print(f"New block #{block_num}")

    block = substr.get_block(block_number=block_num)
    if block is None:
        return

    timestamp = block['extrinsics'][0].value['call']['call_args'][0]['value']
    extr_num = len(block['extrinsics'])

    if gather_baseline:
        baseline_data.loc[block_num] = {
            'size': 0,
            'extrs': extr_num,
            'latency': timestamp - last_timestamp
        }
        print(baseline_data)
        if update_nr == BASELINE_TIMEFRAME - 1:
            gather_baseline = False
            baseline_deviation = baseline_data['extrs'].std()
            print(
                f"Established baseline with {len(baseline_data)} blocks. Current baseline stdev: {baseline_deviation:.3f}"
            )
    else:
        z_score = (
            extr_num - np.mean(baseline_data['extrs'])) / baseline_deviation
        print(f"Z-score of current sample: {z_score:.3f}")

    if z_score > 8 and not capture:
        capture = True
        print("CAPTURE TRIGGERED")

    if capture:
        if z_score < 3:
            end_capture -= 1
        else:
            end_capture = END_CAPTURE_TIMEFRAME

        if end_capture == 0:
            print("END OF CAPTURE")
            return "Done"
        capture_data.loc[block_num] = {  # type: ignore
            'timestamp': timestamp,
            'size': 0,
            'extrs': extr_num,
            'latency': timestamp - last_timestamp
        }
    else:
        print(f"Latency: {(timestamp - last_timestamp) / 1000:.3f}")
        print(f"Number of extrinsics: {extr_num}")
    last_timestamp = timestamp


result = substr.subscribe_block_headers(subscription_handler)

# drop last n entries recorded after the capture should've ended
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
