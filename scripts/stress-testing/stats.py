# NOTE: this script requires the packages numpy, pandas and substrate-interface
# it is recommended to run this in a python venv and install the dependencies with the following commands:
# python3 -m venv venv
# . venv/bin/activate
# pip install -r requirements.txt

import threading
import time
import numpy as np
import pandas as pd
from substrateinterface import SubstrateInterface
from subprocess import Popen, DEVNULL, PIPE, run, TimeoutExpired
from threading import Event, Thread
import shlex
from random import randint
substr = SubstrateInterface(url="wss://1.oracle.network.guild.xyz")
print("Connected")

BLOCK_OVERHEAD = 187  # bytes
END_CAPTURE_TIMEFRAME = 1

# based on 1000 samples
BASELINE = {
    "size_mean": 250.91,
    "size_stdev": 111.4078974741899515,
    "lat_mean": 3.000,
    "lat_stdev": 0.0009411543672990,
    "extr_mean": 1.280,
    "extr_stdev": 0.5864673426065120
}

print(
    f"Baseline stats: {BASELINE}"
)


class Capture:

    def __init__(self, stop_event) -> None:
        self.z_size = np.NaN
        self.last_timestamp = np.NaN
        self.data = pd.DataFrame(
            columns=['timestamp', 'size', 'extrs', 'latency'])
        self.anomalies = {
            "block_time": [],
            "block_size": [],
            "extr_count": [],
        }
        self.failures = 0
        self.stop_event = stop_event
        self.end_capture = END_CAPTURE_TIMEFRAME
        self.capture = False
        self.stats = {
            "t_mean": np.nan,
            "t_stdev": np.nan,
            "s_total": np.nan,
            "s_mean": np.nan,
            "s_stdev": np.nan,
            "e_total": np.nan,
            "e_mean": np.nan,
            "e_stdev": np.nan,
        }

    def __getitem__(self, item):
        return self.data[item]

    def __str__(self) -> str:
        return str(self.data)

    def callback(self, obj):
        block_num = obj['header']['number']
        print(f"New block #{block_num}")

        block = substr.get_block(block_number=block_num)
        if block is None:
            return

        timestamp = block['extrinsics'][0].value['call']['call_args'][0]['value']
        block_size = sum([len(ext.data)
                          for ext in block['extrinsics']]) + BLOCK_OVERHEAD
        extr_num = len(block['extrinsics'])
        latency = (timestamp - self.last_timestamp) / 1000

        self.z_size = (
            block_size - BASELINE["size_mean"]) / BASELINE["size_stdev"]
        z_lat = (latency - BASELINE["lat_mean"]) / BASELINE["lat_stdev"]

        if abs(self.z_size) > 5 and not self.capture:
            self.capture = True
            print("CAPTURE TRIGGERED")
            print(f"z-size: {self.z_size}")

        if self.capture:
            if self.stop_event.is_set():
                if abs(self.z_size) < 3:
                    self.end_capture -= 1
                else:
                    self.end_capture = END_CAPTURE_TIMEFRAME
                if self.end_capture == 0:
                    print("END OF CAPTURE")
                    return "Done"
            self.data.loc[block_num] = {  # type: ignore
                'timestamp': timestamp,
                'size': block_size,
                'extrs': extr_num,
                'latency': latency
            }
            if abs(z_lat) > 3:
                print("ANOMALY IN BLOCKTIME DETECTED")
                print(f"z-lat: {z_lat:.3f}")
                self.anomalies['block_time'].append((latency, z_lat))

            print(f"Latency: {latency:.3f}")
            print(f"Block size: {block_size}")
            print(f"Number of extrinsics: {extr_num}")
        self.last_timestamp = timestamp

    def calc_stats(self):
        self.data.drop(self.data.tail(
            END_CAPTURE_TIMEFRAME-1).index, inplace=True)

        # calculate z-score and modified z-score for each column and add new columns
        for col in ['size', 'extrs', 'latency']:
            self.data['z-' + col] = (self.data[col] -
                                     self.data[col].mean()) / self.data[col].std()
            self.data['z-mod-' + col] = 0.6745 * \
                (self.data[col] - self.data[col].median()) / \
                (self.data[col] - self.data[col].median()).abs().median()

        self.stats = {
            "t_stdev": self.data['latency'].std(),
            "t_mean": self.data['latency'].mean(),
            "s_total": self.data['size'].sum(),
            "s_stdev": self.data['size'].std(),
            "s_mean": self.data['size'].mean(),
            "s_total": self.data['extrs'].sum(),
            "e_stdev": self.data['extrs'].std(),
            "e_mean": self.data['extrs'].mean(),
        }

    def print_stats(self):
        print(self.data)
        print("### Block time")
        if self.data['latency'].std() > BASELINE["lat_stdev"] * 3:
            print(f"  - stdev: {self.stats['t_stdev']:.3f}")
            print(f"  - mean: {self.stats['t_mean']:.3f}")
            print(
                f"  - change in baseline: {self.stats['t_mean']  - BASELINE['lat_mean']:.3f}"
            )
        else:
            print("  - change in baseline: none")

        print("### Block size")
        print(f"  - total stored: {self.data['size'].sum()}")
        print(f"  - stdev: {self.stats['s_stdev']:.3f}")
        print(f"  - mean: {self.stats['s_mean']:.3f}")
        print(
            f"  - change in baseline: {self.stats['s_mean']  - BASELINE['size_mean']:.3f}"
        )

        print("### Extrinsics")
        print(f"  - total executed: {self.data['extrs'].sum()}")
        print(
            f"  - extrinsics per second: {self.data['extrs'].mean() * 3:.3f}")
        print(f"  - stdev: {self.stats['e_stdev']:.3f}")
        print(f"  - mean: {self.stats['e_mean']:.3f}")
        print(
            f"  - change in baseline: {self.stats['e_mean']  - BASELINE['extr_mean']:.3f}"
        )
        print(f"Test lasted {len(self.data.index)} blocks")


def start_collection(stop_event: Event):

    capture = Capture(stop_event)

    def subscription_handler(obj, _update_nr, _subscription_id):
        return capture.callback(obj)

    substr.subscribe_block_headers(subscription_handler)

    capture.calc_stats()
    capture.print_stats()
    return capture


def single_test(tps, num, stop_event: Event):
    cmd = f"../../target/release/gn-cli -i 65.108.102.250 stress --seed {hex(randint(0, 0xffffffff))} --tps {tps} -n {num} register-other"
    test = Popen(shlex.split(cmd), stderr=PIPE, stdout=DEVNULL)
    test.wait()
    stop_event.set()


def start_iteration(tps, num):
    stop_event = threading.Event()
    test = Thread(target=single_test, args=(tps, num, stop_event))
    test.start()
    return start_collection(stop_event)


def start_tests(params):
    results = []
    for _ in range(0, 20):
        res = start_iteration(64, params)
        results.append(res)


def main():
    initial_parameters = 256
    start_tests(initial_parameters)


if __name__ == "__main__":
    main()