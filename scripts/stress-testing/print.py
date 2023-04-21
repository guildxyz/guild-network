# NOTE: this script requires the packages numpy, pandas and substrate-interface
# it is recommended to run this in a python venv and install the dependencies with the following commands:
# python3 -m venv venv
# . venv/bin/activate
# pip install -r requirements.txt

import time
import numpy as np
import pandas as pd

capture_data = pd.read_csv("baseline.csv", sep=";")
print(capture_data)

capture_data['z-lat'] = (capture_data['latency'] - capture_data['latency'].mean()
                         ) / capture_data['latency'].std()
capture_data['z-size'] = (capture_data['latency'] - capture_data['latency'].mean()
                          ) / capture_data['latency'].std()
capture_data.to_csv("baseline.csv", sep=";")
print(capture_data)

print(f"latency mean: {capture_data['latency'].mean():.16f}")
print(f"latency stdev: {capture_data['latency'].std():.16f}")

print(f"block size total: {capture_data['size'].sum()}")
print(f"block size mean: {capture_data['size'].mean():.16f}")
print(f"block size stdev: {capture_data['size'].std():.16f}")

print(f"extrinsic count: {capture_data['extrs'].sum()}")
print(f"extrinsic mean: {capture_data['extrs'].mean():.16f}")
print(f"extrinsic stdev: {capture_data['extrs'].std():.16f}")
print(
    f"extrinsics per second: {capture_data['extrs'].sum() / (capture_data['extrs'].count() * 3):.16f}"
)

print(f"Test lasted {len(capture_data.index)} blocks")
