# Initial load test results

## Baseline condictions (in 250 blocks)
### Block time
- avg block time: 3s
- avg stdev: 0.001 (+/-0.01)
### Block size
- avg block size: 255.776 bytes
### Extrensics
- avg number of extrensics: 1.316

## Test 1
- Number of tx sent: 256
- TPS: 256

### Block time
  - change in baseline: none
### Block size
  - total stored: **91532**
  - stdev: 21360.282
  - mean: 45766.000
  - change in baseline: 45510.224
### Extrinsics
  - total executed: **514**
  - extrinsics per second: 771.000
  - stdev: 0.000
  - mean: 257.000
  - change in baseline: 255.684

#### Results
- Number of blocks executed during test: 2

- Outstanding observations: For all extrinsics sent, the oracle replies with the same amount of extrinsics, which results in the block size effectively doubling every extrinsic sent as highlighted above.

- Conclusion: No significant observable change in baseline blocktime.


## Test 2
- Number of tx sent: 512
- TPS: 256

### Block time
  - change in baseline: none
### Block size
  - total stored: 183562
  - stdev: 30337.500
  - mean: 61187.333
  - change in baseline: 60931.557
### Extrinsics
  - total executed: 1031
  - extrinsics per second: 1031.000
  - stdev: 147.514
  - mean: 343.667
  - change in baseline: 342.351

### Results
- Number of blocks executed during test: 3

- Outstanding observations: 
  - During one of the iterations of this test, a node seemingly crashed for a few authoring rounds. The cause of this is unknown and not consistently reproducible.
  - In some iterations of this test, for some unknown reason the roughly 2/3rd of the extrinsics don't show up at all in blocks. This could be a bug in the explorer, since I was unable to consistently reproduce these results.

- Conclusion: No significant observable change in baseline blocktime, average block size doubled as expected.


## Test 3
- Number of tx sent: 768
- TPS: 256

### Block time
  - change in baseline: none
### Block size
  - total stored: 274200
  - stdev: 29055.718
  - mean: 68550.000
  - change in baseline: 68294.224
### Extrinsics
  - total executed: 1540
  - extrinsics per second: 1155.000
  - stdev: 147.802
  - mean: 385.000
  - change in baseline: 383.684

  
### Results
- Number of blocks executed during test: 4

- Outstanding observations: none

- Conclusion: No significant observable change in baseline blocktime, increase in average block size is of a factor 2.99 as expected (with Test 1 as baseline), increase in total amount of extrinsics is as expected.


## Test 4
- Number of tx sent: 2048
- TPS: 256

### Block time
  - change in baseline: none
### Block size
  - total stored: 517028
  - stdev: 67585.826
  - mean: 86171.333
  - change in baseline: 85915.557
### Extrinsics
  - total executed: 2310
  - extrinsics per second: 1155.000
  - stdev: 268.495
  - mean: 385.000
  - change in baseline: 383.684

### Results
- Number of blocks executed during test: 6

- Outstanding observations: The block size increased by a factor of 1.88 instead of the expected increase of a factor of 2.66 since the last test. This is possibly caused by the fact that the oracle doubles the number of extrinsics executed per block, although this effect couldn't be observed in previous tests.

- Conclusion: No significant observable change in baseline blocktime, increase in average block size falls short of the expected increase by a factor of 0.7.

## Test 5
- Number of tx sent: 4096
- TPS: 512 (256 on 2 threads)

### Block time
  - stdev: 0.003
  - mean: 3.000
  - change in baseline: 0.000
### Block size
  - total stored: 972283
  - stdev: 59262.784
  - mean: 194456.600
  - change in baseline: 194200.824
### Extrinsics
  - total executed: 4104
  - extrinsics per second: 2462.400
  - stdev: 250.051
  - mean: 820.800
  - change in baseline: 819.484

### Results
- Number of blocks executed during test: 5

- Outstanding observations: 
  - An increase of 0.002 sigma has been observed in the standard deviation of the blocktime during the test.
  - The block size increased by a factor of 1.88 instead of the expected increase of a factor of 2 since the last test. 
  - Similarly, the number of extrinsics executed has incresed by a factor of 1.77 instead the expected increase of a factor of 2 since the last test.

- Conclusion: No significant observable change in baseline blocktime, although anomalies have been observed in the dispersion. Increase in average block size falls short of the expected increase by a factor of 0.885.

## Test 6
- Number of tx sent: 8192
- TPS: 512 (256 on 2 threads)

### Block time
  - change in baseline: none
### Block size
  - total stored: 1943870
  - stdev: 94752.229
  - mean: 194387.000
  - change in baseline: 194131.224
### Extrinsics
  - total executed: 8204
  - extrinsics per second: 2461.200
  - stdev: 399.800
  - mean: 820.400
  - change in baseline: 819.084

### Results
- Number of blocks executed during test: 10

- Outstanding observations: 
  - There was one block "skipped" in the execution of extrinsics. The cause of this behavior is unknown.

- Conclusion: No significant observable change in baseline blocktime, block size and extrinsic count has increased by a factor of 2 as expected.


## Test 7
- Number of tx sent: 8192
- TPS: 1024 (256 on 4 threads)


### Block time
  - change in baseline: none
### Block size
  - total stored: 1942877
  - stdev: 101499.941
  - mean: 388575.400
  - change in baseline: 388319.624
### Extrinsics
  - total executed: 8199
  - extrinsics per second: 4919.400
  - stdev: 428.251
  - mean: 1639.800
  - change in baseline: 1638.484

### Results
- Number of blocks executed during test: 5

- Outstanding observations: none

- Conclusion: No significant observable change in baseline blocktime, block size and extrinsic count has remained the same as expected, while execution time has halved and mean values doubled.


## Test 8
- Number of tx sent: 8192
- TPS: 2048 (256 on 8 threads)

### Iteration 1
#### Block time
  - change in baseline: none
#### Block size
  - total stored: 1942494
  - stdev: 291487.821
  - mean: 388498.800
  - change in baseline: 388243.024
#### Extrinsics
  - total executed: 8197
  - extrinsics per second: 4918.200
  - stdev: 1229.906
  - mean: 1639.400
  - change in baseline: 1638.084

### Iteration 2
#### Block time
  - stdev: 1.732
  - mean: 4.500
  - change in baseline: 1.500
#### Block size
  - total stored: 1942646
  - stdev: 126099.276
  - mean: 485661.500
  - change in baseline: 485405.724
#### Extrinsics
  - total executed: 8198
  - extrinsics per second: 6148.500
  - stdev: 531.938
  - mean: 2049.500
  - change in baseline: 2048.184

### Iteration 3
#### Block time
  - stdev: 1.549
  - mean: 4.000
  - change in baseline: 1.000
#### Block size
  - total stored: 1943292
  - stdev: 305817.321
  - mean: 323882.000
  - change in baseline: 323626.224
#### Extrinsics
  - total executed: 8201
  - extrinsics per second: 4100.500
  - stdev: 1290.269
  - mean: 1366.833
  - change in baseline: 1365.517

### Results
- Number of blocks executed during test: 4-6

- Outstanding observations:
  - This test has produced inconsistent results across multiple iterations (see some outstanding records above). This might be a sign of instability in the functioning of the individual nodes, the underlying hardware, or the chain in its entirety.
  - In some iterations of this test, the mean block time and dispersion has deviated significantly from the baseline, which might indicate the the temporary crash of an authoring node's runtime during the test. 
  - Block size mean and extrinsics mean between the last test has varied between iterations of this test ranging from a factor of 0.833 to 1.25 instead of the expected factor of 2. This might indicate that the chain's computational capacity has been exceeeded by the eps (extrensics per second) used in the test.
  - Execution time has also varied between multiple iterations of this test, and is negligible compared to the expected decrease of 50% since the last test.

- Conclusion: **Significant change in both the mean and dispersion of the block time has been observed**, block size and extrinsic count has remained the same as expected, but increase in block size and extrinsics mean has fallen short of the expected amount; see a detailed explanation of results above.

# New testing framework

## Initial testing

The purpose of this test is to establish an estimated range of load conditions
under which the chain can no longer function properly.

### Round 1
Conditions: Continuous flood stress with increasing pressure
Initial parameters:
  - 256 tps
  - 256 total tx
  - 10 iterations per cycle
  - 256 increase per cycle
See round1.log

END TEST CYCLE WITH PARAMETERS 1792 num, 1792 tps; FAILURES DETECTED: 3, RATE: 30%

### Round 2
Conditions: Continuous flood stress with increasing pressure
Initial parameters:
    - 1000 tps
    - 1000 total tx
    - 20 iterations per cycle
    - 100 increase per cycle
  See round2.log

END TEST CYCLE WITH PARAMETERS 1200 num, 1200 tps; FAILURES DETECTED: 1, RATE: 5.0%
END TEST CYCLE WITH PARAMETERS 1300 num, 1300 tps; FAILURES DETECTED: 2, RATE: 10.0%
END TEST CYCLE WITH PARAMETERS 1400 num, 1400 tps; FAILURES DETECTED: 3, RATE: 15.0%
END TEST CYCLE WITH PARAMETERS 1500 num, 1500 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 1600 num, 1600 tps; FAILURES DETECTED: 7, RATE: 35.0%
END TEST CYCLE WITH PARAMETERS 1700 num, 1700 tps; FAILURES DETECTED: 5, RATE: 25.0%
END TEST CYCLE WITH PARAMETERS 1800 num, 1800 tps; FAILURES DETECTED: 3, RATE: 15.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 1900 tps; FAILURES DETECTED: 10, RATE: 50.0%

### Round 3
Conditions: Continuous flood stress with increasing pressure
Initial parameters:
    - 1900 tps
    - 1900 total tx
    - 20 iterations per cycle
    - 100 increase per cycle
  See round3.log

END TEST CYCLE WITH PARAMETERS 1900 num, 1900 tps; FAILURES DETECTED: 4, RATE: 20.0%
END TEST CYCLE WITH PARAMETERS 2000 num, 2000 tps; FAILURES DETECTED: 12, RATE: 60.0%

Test aborted unexpectedly due issues with local node.

### Round 4
Conditions: Continuous flood stress with increasing pressure
Initial parameters:
    - 2100 tps
    - 2100 total tx
    - 20 iterations per cycle
    - 100 increase per cycle
  See round4.log

END TEST CYCLE WITH PARAMETERS 2100 num, 2100 tps; FAILURES DETECTED: 8, RATE: 40.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 14, RATE: 70.0%
END TEST CYCLE WITH PARAMETERS 2300 num, 2300 tps; FAILURES DETECTED: 20, RATE: 100.0%

Test aborted prematurely due to 100% failure rate in block time


### Conclusion for initial testing

A failure rate of 5% was observed under constant extended load (1200 tps) which
then continually increased with pressure. Further testing to verify these results is required.

## Reliability testing
 
The purpose of this test is to further examine and refine the upper bounds of
load conditions under which the chain can no longer function properly.

### Round 1

Conditions: Increasing pressure with constant load
Parameters:
  - 100 tps
  - 1900 total tx
  - 20 iterations per cycle
  - 100 increase in tps per cycle

See reliability.log

END TEST CYCLE WITH PARAMETERS 1900 num, 500 tps; FAILURES DETECTED: 1, RATE: 5.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 600 tps; FAILURES DETECTED: 7, RATE: 35.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 700 tps; FAILURES DETECTED: 10, RATE: 50.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 800 tps; FAILURES DETECTED: 4, RATE: 20.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 900 tps; FAILURES DETECTED: 13, RATE: 65.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 1000 tps; FAILURES DETECTED: 12, RATE: 60.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 1100 tps; FAILURES DETECTED: 15, RATE: 75.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 1200 tps; FAILURES DETECTED: 9, RATE: 45.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 1300 tps; FAILURES DETECTED: 9, RATE: 45.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 1400 tps; FAILURES DETECTED: 10, RATE: 50.0%
END TEST CYCLE WITH PARAMETERS 1900 num, 1500 tps; FAILURES DETECTED: 11, RATE: 55.00000000000001%
END TEST CYCLE WITH PARAMETERS 1900 num, 1600 tps; FAILURES DETECTED: 11, RATE: 55.00000000000001%
END TEST CYCLE WITH PARAMETERS 1900 num, 1700 tps; FAILURES DETECTED: 10, RATE: 50.0%

### Round 2

Conditions: Constant low pressure with constant moderate load
Parameters:
  - 10 tps
  - 500 total tx
  - 100 iterations per cycle

END TEST CYCLE WITH PARAMETERS 500 num, 10 tps; FAILURES DETECTED: 0, RATE: 0.0%

### Round 3

Conditions: Constant high pressure with increasing load

Parameters:
  - 500 tps
  - 10 total tx
  - 25 iterations per cycle
  - 10 increase in total tx per cycle
  - up until 280 total tx
 
No failures observed 

### Round 4

Conditions: Constant low pressure, constant high load, 100 rounds

Parameters:
  - 500 tps
  - 2000 total tx
  - 100 iterations

END TEST CYCLE WITH PARAMETERS 2000 num, 500 tps; FAILURES DETECTED: 10, RATE: 10.0%

### Round 5

Conditions: Constant low pressure, constant high load, 100 rounds

Parameters:
  - 500 tps
  - 1000 total tx
  - 100 iterations

END TEST CYCLE WITH PARAMETERS 1000 num, 500 tps; FAILURES DETECTED: 0, RATE: 0.0%

### Round 6

Conditions: Constant low pressure, constant high load, 100 rounds

Parameters:
  - 500 tps
  - 1500 total tx
  - 100 iterations

END TEST CYCLE WITH PARAMETERS 1500 num, 500 tps; FAILURES DETECTED: 13, RATE: 13.0%

### Round 7

Conditions: Constant high pressure, constant high load, 100 rounds

Parameters:
  - 1200 tps
  - 1200 total tx
  - 100 iterations

END TEST CYCLE WITH PARAMETERS 1200 num, 1200 tps; FAILURES DETECTED: 0, RATE: 0.0%

### Round 8

Conditions: Constant high pressure, constant high load, 100 rounds

Parameters:
  - 1900 tps
  - 1900 total tx
  - 100 iterations

END TEST CYCLE WITH PARAMETERS 1900 num, 1900 tps; FAILURES DETECTED: 4, RATE: 4.0%

### Round 9

Conditions: Constant high pressure, constant high load, 100 rounds

Parameters:
  - 2200 tps
  - 2200 total tx
  - 100 iterations
