# NOTE: this script requires the packages numpy, pandas and substrate-interface
# it is recommended to run this in a python venv and install the dependencies with the following commands:
# python3 -m venv venv
# . venv/bin/activate
# pip install -r requirements.txt

import threading
import time
from typing import List
import numpy as np
import pandas as pd
from substrateinterface import SubstrateInterface
from subprocess import Popen, DEVNULL, PIPE, run, TimeoutExpired
from threading import Event, Thread
import shlex
from random import randint
import scipy.stats

substr = SubstrateInterface(url="ws://localhost:9944")
print("Connected")

BLOCK_OVERHEAD = 187  # bytes
END_CAPTURE_TIMEFRAME = 1
TEST_CYCLE = 25
SIG_FAIL_LVL = 0.05  # 5%

# based on 1000 samples
BASELINE = {
    "size_mean": 250.91,
    "size_stdev": 111.4078974741899515,
    "lat_mean": 3.000,
    "lat_stdev": 0.0009411543672990,
    "extr_mean": 1.280,
    "extr_stdev": 0.5864673426065120
}

template = """
FAILURE
  - mean: {:.2f}
  - unbiased stdev: {:.5f}
  - unbiased sterr: {:.5f}
  - margin of error (95% conf. lvl.) = +/-{:.3f}

RATE:
  - mean: {:.2f}%
  - unbiased stdev: {:.5f}%
  - unbiased sterr: {:.5f}%
  - margin of error (95% conf. lvl.) = +/-{:.3f}%
"""

print(
    f"Baseline stats: {BASELINE}"
)


class SignificantFailure(Exception):
    pass


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
            # print("CAPTURE TRIGGERED")
            print(f"z-size: {self.z_size}")

        if self.capture:
            if self.stop_event.is_set():
                if abs(self.z_size) < 3:
                    self.end_capture -= 1
                else:
                    self.end_capture = END_CAPTURE_TIMEFRAME
                if self.end_capture == 0:
                    return "Done"
            self.data.loc[block_num] = {  # type: ignore
                'timestamp': timestamp,
                'size': block_size,
                'extrs': extr_num,
                'latency': latency
            }
            if z_lat > 3134:
                self.failures += 1

            if abs(z_lat) > 3:
                print("ANOMALY IN BLOCKTIME DETECTED")
                print(f"z-lat: {z_lat:.3f}")
                self.anomalies['block_time'].append(z_lat)

            print(f"Latency: {latency:.3f}")
            # print(f"Block size: {block_size}")
            print(f"Number of extrinsics: {extr_num}")
        self.last_timestamp = timestamp

    def calc_stats(self):
        self.data.drop(self.data.tail(
            END_CAPTURE_TIMEFRAME-1).index, inplace=True)

        # calculate z-score and modified z-score for each column and add new columns
        for col in ['size', 'extrs', 'latency']:
            self.data['z-' + col] = (self.data[col] -
                                     self.data[col].mean()) / self.data[col].std()

        self.stats = {
            "t_stdev": self.data['latency'].std(),
            "t_mean": self.data['latency'].mean(),
            "s_total": self.data['size'].sum(),
            "s_stdev": self.data['size'].std(),
            "s_mean": self.data['size'].mean(),
            "e_total": self.data['extrs'].sum(),
            "e_stdev": self.data['extrs'].std(),
            "e_mean": self.data['extrs'].mean(),
        }

    def print_stats(self):
        print(self.stats)

    def pretty_print_stats(self):
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


class CaptureCycle:
    def __init__(self, tps, tx_num) -> None:
        self.tps = tps
        self.tx_num = tx_num
        self.captures = {}

    def start_collection(self, iter_n, stop_event: Event):

        capture = Capture(stop_event)

        def subscription_handler(obj, _update_nr, _subscription_id):
            return capture.callback(obj)

        substr.subscribe_block_headers(subscription_handler)

        capture.calc_stats()
        capture.print_stats()
        self.captures[iter_n] = capture
        return capture

    def single_test(self, iter_n, tps, tx_num, stop_event: Event):
        print(
            f"Start iteration {iter_n+1}, with params {tx_num} num, {tps} tps")
        cmd = f"../../target/release/gn-cli -i localhost stress --seed {hex(randint(0, 0xffffffff))} --tps {tps} -n {tx_num} register-other"
        print("START TEST")
        test = Popen(shlex.split(cmd), stderr=PIPE, stdout=PIPE)

        print("WAITING")
        test.wait()
        print("END WAIT")
        stop_event.set()

    def start_iteration(self, iter_n):
        stop_event = threading.Event()
        test = Thread(target=self.single_test, args=(
            iter_n, self.tps, self.tx_num, stop_event))
        test.start()
        return self.start_collection(iter_n, stop_event)

    def count_failures(self, test_cycle: List[Capture]):
        failures = 0

        for iteration in test_cycle:
            failures += iteration.failures
        return failures

    def start_tests(self, n):
        results = []
        print(
            f"START TEST CYCLE {n} WITH PARAMETERS {self.tx_num} num, {self.tps} tps")
        for i in range(0, TEST_CYCLE):
            res = self.start_iteration(i)
            results.append(res)
        failures = self.count_failures(results)
        print(
            f"END TEST CYCLE {n} WITH PARAMETERS {self.tx_num} num, {self.tps} tps; FAILURES DETECTED: {failures}, RATE: {failures/TEST_CYCLE*100}%"
        )

        if failures >= int(TEST_CYCLE * SIG_FAIL_LVL):
            print(f"SIGNIFICANT FAILURE DETECTED AT {self.tx_num}")
        return failures/TEST_CYCLE


def initial_testing():
    params = 2100
    results = {}
    capture_cycles = []
    while True:
        c = CaptureCycle(params, params)
        failure_rate = c.start_tests()
        results[params] = failure_rate
        if failure_rate >= int(TEST_CYCLE * 0.5):
            break
        params += 100
    print(
        f"First significant failure detected at {list(results.keys())[0]} num, {list(results.keys())[0]} tps")
    print(f"")
    print(results)
    print(capture_cycles)


def print_result_stats(results):
    rates = list(results.values())
    fails = [int(i * TEST_CYCLE) for i in rates]
#
    mu_f = np.mean(fails)
    sigma_f = np.std(fails, ddof=1.5)
    sem_f = scipy.stats.sem(fails, ddof=1.5)
    moe_f = 1.96 * sem_f
#
    mu_r = np.mean(rates)
    sigma_r = np.std(rates, ddof=1.5)
    sem_r = scipy.stats.sem(rates, ddof=1.5)
    moe_r = 1.96 * sem_r
#
    print("Fails:", fails)
    print("Rates:", rates)
    print(template.format(mu_f, sigma_f, sem_f, moe_f,
                          mu_r * 100, sigma_r * 100, sem_r * 100, moe_r*100))


def reliability_testing():
    params = 1875
    results = {}
    capture_cycles = []
    # while True:
    for i in range(20):
        c = CaptureCycle(params, params)
        failure_rate = c.start_tests(i)
        results[i] = failure_rate
        print(f"Capture cycle {i} ended")
    # if params == 500:
    # break
    # params += 10
    print_result_stats(results)


def main():
    reliability_testing()


if __name__ == "__main__":
    main()
