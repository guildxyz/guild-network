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

**NOTE:** this, and possibly many of the previous tests has been run without a
working oracle, rerunning these tests may be necessary to confirm the accuracy
of these results

Conditions: Constant high pressure, constant high load, 100 rounds

Parameters:
  - 2200 tps
  - 2200 total tx
  - 100 iterations
  - 5 cycles

w/o working oracle: END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 43, RATE: 43.0%
w/  working oracle:
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 35, RATE: 35.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 38, RATE: 38.0%

mean: 36.5%
cor. stdev: 2.12132%
cor. stderr: 1.5%
margin of error (95% conf. lvl.) = +/-2.925%

### Round 10

Condition: Constant high pressure, constant high load

Parameters:
  - 2200 tps
  - 2200 total tx
  - 25 testing cycle
  - 20 rounds per cycle


END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 7, RATE: 35.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 8, RATE: 40.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 4, RATE: 20.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 5, RATE: 25.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 8, RATE: 40.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 8, RATE: 40.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 7, RATE: 35.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 5, RATE: 25.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 8, RATE: 40.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 9, RATE: 45.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 7, RATE: 35.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 9, RATE: 45.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 9, RATE: 45.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 8, RATE: 40.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 5, RATE: 25.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 5, RATE: 25.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 6, RATE: 30.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 7, RATE: 35.0%
END TEST CYCLE WITH PARAMETERS 2200 num, 2200 tps; FAILURES DETECTED: 8, RATE: 40.0%


FAILURE
  - mean: 6.76
  - unbiased stdev: 1.43749
  - unbiased sterr: 0.28750
  - margin of error (95% conf. lvl.) = +/-0.563

RATE:
  - mean: 33.8%
  - unbiased stdev: 7.18746%
  - unbiased sterr: 1.43749%
  - margin of error (95% conf. lvl.) = +/-2.817%

### Round 11

Condition: Constant moderate pressure, constant high load

Parameters:
  - 1900 tps
  - 1900 total tx
  - 25 testing cycle
  - 20 rounds per cycle


[10, 8, 5, 7, 7, 6, 9, 9, 10, 7, 5, 8, 7, 5, 9, 7, 6, 6, 9, 8, 6, 7, 6, 8, 8]
[50, 40, 25, 35, 35, 30, 45, 45, 50, 35, 25, 40, 35, 25, 45, 35, 30, 30, 45, 40, 30, 35, 30, 40, 40]


FAILURE
  - mean: 7.32
  - unbiased stdev: 1.50799
  - unbiased sterr: 0.30159
  - margin of error (95% conf. lvl.) = +/-0.563

RATE:
  - mean: 36.6%
  - unbiased stdev: 7.53996%
  - unbiased sterr: 1.50799%
  - margin of error (95% conf. lvl.) = +/-2.953%

### Round 12

Condition: Constant moderate pressure, constant high load

Parameters:
  - 1900 tps
  - 1900 total tx
  - 10 testing cycle
  - 10 rounds per cycle

Fails: [1, 5, 3, 3, 4, 4, 5, 5, 2, 3]
Rates: [0.1, 0.5, 0.3, 0.3, 0.4, 0.4, 0.5, 0.5, 0.2, 0.3]

FAILURE
  - mean: 3.50
  - unbiased stdev: 1.39326
  - unbiased sterr: 0.44059
  - margin of error (95% conf. lvl.) = +/-0.864

RATE:
  - mean: 35.00%
  - unbiased stdev: 13.93261%
  - unbiased sterr: 4.40588%
  - margin of error (95% conf. lvl.) = +/-8.636%

### Round 13

Condition: Constant moderate pressure, constant high load

Parameters:
  - 1200 tps
  - 1200 total tx
  - 10 testing cycle
  - 10 rounds per cycle

Fails: [2, 2, 2, 2, 2, 3, 2, 2, 2, 3]
Rates: [0.2, 0.2, 0.2, 0.2, 0.2, 0.3, 0.2, 0.2, 0.2, 0.3]

FAILURE
  - mean: 2.20
  - unbiased stdev: 0.43386
  - unbiased sterr: 0.13720
  - margin of error (95% conf. lvl.) = +/-0.269

RATE:
  - mean: 22.00%
  - unbiased stdev: 4.33861%
  - unbiased sterr: 1.37199%
  - margin of error (95% conf. lvl.) = +/-2.689%

### Round 13

Condition: Constant moderate pressure, constant high load

Parameters:
  - 1200 tps
  - 1200 total tx
  - 10 testing cycle
  - 10 rounds per cycle

Fails: [2, 2, 2, 2, 2, 3, 2, 2, 2, 3]
Rates: [0.2, 0.2, 0.2, 0.2, 0.2, 0.3, 0.2, 0.2, 0.2, 0.3]

FAILURE
  - mean: 2.20
  - unbiased stdev: 0.43386
  - unbiased sterr: 0.13720
  - margin of error (95% conf. lvl.) = +/-0.269

RATE:
  - mean: 22.00%
  - unbiased stdev: 4.33861%
  - unbiased sterr: 1.37199%
  - margin of error (95% conf. lvl.) = +/-2.689%

### Round 13

Condition: Constant moderate pressure, constant high load

Parameters:
  - 1200 tps
  - 1200 total tx
  - 20 testing cycle
  - 5 rounds per cycle

Fails: [3, 6, 5, 3, 3]
Rates: [0.15, 0.3, 0.25, 0.15, 0.15]

FAILURE
  - mean: 4.00
  - unbiased stdev: 1.51186
  - unbiased sterr: 0.67612
  - margin of error (95% conf. lvl.) = +/-1.325

RATE:
  - mean: 20.00%
  - unbiased stdev: 7.55929%
  - unbiased sterr: 3.38062%
  - margin of error (95% conf. lvl.) = +/-6.626%


### Round 14

Condition: Constant moderate pressure, constant high load

Parameters:
  - 1000 tps
  - 1000 total tx
  - 20 testing cycle
  - 10 rounds per cycle

Fails: [2, 0, 0, 0, 4, 3, 2, 5, 2, 5]
Rates: [0.1, 0.0, 0.0, 0.0, 0.2, 0.15, 0.1, 0.25, 0.1, 0.25]

FAILURE
  - mean: 2.30
  - unbiased stdev: 2.00294
  - unbiased sterr: 0.63338
  - margin of error (95% conf. lvl.) = +/-1.241

RATE:
  - mean: 11.50%
  - unbiased stdev: 10.01470%
  - unbiased sterr: 3.16692%
  - margin of error (95% conf. lvl.) = +/-6.207%

### Round 15

Condition: Constant moderate pressure, constant high load

Parameters:
  - 1000 tps
  - 1000 total tx
  - 15 testing cycle
  - 15 rounds per cycle

Fails: [3, 5, 2, 2, 4, 3, 2, 3, 0, 1, 1, 1, 0, 2, 1]
Rates: [0.2, 0.3333333333333333, 0.13333333333333333, 0.13333333333333333, 0.26666666666666666, 0.2, 0.13333333333333333, 0.2, 0.0, 0.06666666666666667, 0.06666666666666667, 0.06666666666666667, 0.0, 0.13333333333333333, 0.06666666666666667]

FAILURE
  - mean: 2.00
  - unbiased stdev: 1.44016
  - unbiased sterr: 0.37185
  - margin of error (95% conf. lvl.) = +/-0.729

RATE:
  - mean: 13.33%
  - unbiased stdev: 9.60110%
  - unbiased sterr: 2.47899%
  - margin of error (95% conf. lvl.) = +/-4.859%

### Round 15

Condition: Constant moderate pressure, constant high load

Parameters:
  - 900 tps
  - 900 total tx
  - 15 testing cycle
  - 15 rounds per cycle

Fails: [1, 0, 1, 0, 0, 0, 0, 1, 0, 1, 2, 1, 1, 0, 0]
Rates: [0.06666666666666667, 0.0, 0.06666666666666667, 0.0, 0.0, 0.0, 0.0, 0.06666666666666667, 0.0, 0.06666666666666667, 0.13333333333333333, 0.06666666666666667, 0.06666666666666667, 0.0, 0.0]

FAILURE
  - mean: 0.53
  - unbiased stdev: 0.65168
  - unbiased sterr: 0.16826
  - margin of error (95% conf. lvl.) = +/-0.330

RATE:
  - mean: 3.56%
  - unbiased stdev: 4.34456%
  - unbiased sterr: 1.12176%
  - margin of error (95% conf. lvl.) = +/-2.199%

### Round 16

Condition: Constant moderate pressure, constant high load

Parameters:
  - 950 tps
  - 950 total tx
  - 15 testing cycle
  - 15 rounds per cycle

Fails: [1, 0, 1, 1, 0, 1, 0, 1, 4, 0, 0, 1, 2, 0, 0]
Rates: [0.06666666666666667, 0.0, 0.06666666666666667, 0.06666666666666667, 0.0, 0.06666666666666667, 0.0, 0.06666666666666667, 0.26666666666666666, 0.0, 0.0, 0.06666666666666667, 0.13333333333333333, 0.0, 0.0]

FAILURE
  - mean: 0.80
  - unbiased stdev: 1.10219
  - unbiased sterr: 0.28458
  - margin of error (95% conf. lvl.) = +/-0.558

RATE:
  - mean: 5.33%
  - unbiased stdev: 7.34791%
  - unbiased sterr: 1.89722%
  - margin of error (95% conf. lvl.) = +/-3.719%

### Round 16

Condition: Constant moderate pressure, constant high load

Parameters:
  - 950 tps
  - 950 total tx
  - 15 testing cycle
  - 15 rounds per cycle

Fails: [0, 0, 0, 0, 0, 0, 1, 0, 1, 2, 0, 0, 1, 0, 0]
Rates: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.06666666666666667, 0.0, 0.06666666666666667, 0.13333333333333333, 0.0, 0.0, 0.06666666666666667, 0.0, 0.0]

FAILURE
  - mean: 0.33
  - unbiased stdev: 0.62854
  - unbiased sterr: 0.16229
  - margin of error (95% conf. lvl.) = +/-0.318

RATE:
  - mean: 2.22%
  - unbiased stdev: 4.19026%
  - unbiased sterr: 1.08192%
  - margin of error (95% conf. lvl.) = +/-2.121%

###  Round 17

Condition: Constant moderate pressure, constant high load

Parameters:
  - 950 tps
  - 950 total tx
  - 20 testing cycle
  - 25 rounds per cycle


Fails: [1, 4, 1, 3, 0, 1, 0, 2, 2, 1, 3, 1, 4, 1, 1, 0, 1, 1, 0, 0]
Rates: [0.04, 0.16, 0.04, 0.12, 0.0, 0.04, 0.0, 0.08, 0.08, 0.04, 0.12, 0.04, 0.16, 0.04, 0.04, 0.0, 0.04, 0.04, 0.0, 0.0]

FAILURE
  - mean: 1.35
  - unbiased stdev: 1.28505
  - unbiased sterr: 0.28735
  - margin of error (95% conf. lvl.) = +/-0.563

RATE:
  - mean: 5.40%
  - unbiased stdev: 5.14020%
  - unbiased sterr: 1.14938%
  - margin of error (95% conf. lvl.) = +/-2.253%

###  Round 18

*Note: parameters are set for an unsupervised overnight test*

Condition: Constant moderate pressure, constant high load

Parameters:
  - 950 tps
  - 950 total tx
  - 50 testing cycle
  - 50 rounds per cycle

Fails: [2, 2, 4, 2, 0, 4, 0, 2, 1, 3, 1, 2, 4, 1, 2, 2, 2, 2, 3, 0, 5, 2, 3, 1, 3, 1, 1, 1, 1, 2, 2, 4, 3, 0, 2, 2, 2, 1, 2, 3, 2, 1, 1, 3, 1, 1, 2, 4, 2, 2]
Rates: [0.04, 0.04, 0.08, 0.04, 0.0, 0.08, 0.0, 0.04, 0.02, 0.06, 0.02, 0.04, 0.08, 0.02, 0.04, 0.04, 0.04, 0.04, 0.06, 0.0, 0.1, 0.04, 0.06, 0.02, 0.06, 0.02, 0.02, 0.02, 0.02, 0.04, 0.04, 0.08, 0.06, 0.0, 0.04, 0.04, 0.04, 0.02, 0.04, 0.06, 0.04, 0.02, 0.02, 0.06, 0.02, 0.02, 0.04, 0.08, 0.04, 0.04]

FAILURE
  - mean: 1.98
  - unbiased stdev: 1.15749
  - unbiased sterr: 0.16369
  - margin of error (95% conf. lvl.) = +/-0.321

RATE:
  - mean: 3.96%
  - unbiased stdev: 2.31499%
  - unbiased sterr: 0.32739%
  - margin of error (95% conf. lvl.) = +/-0.642%

### Round 19

Condition: Constant moderate pressure, constant high load

Parameters:
  - 2800 tps
  - 2800 total tx
  - 15 testing cycle
  - 15 rounds per cycle

Fails: [20, 16, 13, 11, 17, 18, 16, 14, 14, 11, 9, 18, 14, 15, 12]
Rates: [1.3333333333333333, 1.0666666666666667, 0.8666666666666667, 0.7333333333333333, 1.1333333333333333, 1.2, 1.0666666666666667, 0.9333333333333333, 0.9333333333333333, 0.7333333333333333, 0.6, 1.2, 0.9333333333333333, 1.0, 0.8]

FAILURE
  - mean: 14.53
  - unbiased stdev: 3.09998
  - unbiased sterr: 0.80041
  - margin of error (95% conf. lvl.) = +/-1.569

RATE:
  - mean: 96.89%
  - unbiased stdev: 20.66653%
  - unbiased sterr: 5.33608%
  - margin of error (95% conf. lvl.) = +/-10.459%

MOE  

### Round 20

Condition: Constant moderate pressure, constant high load

Parameters:
  - 2800 tps
  - 2800 total tx
  - 20 testing cycle
  - 25 rounds per cycle

Fails: [16, 17, 17, 20, 15, 17, 17, 32, 16, 14, 14, 15, 13, 11, 13, 11, 9, 9, 9, 13, 10, 12, 11, 14, 14]
Rates: [0.8, 0.85, 0.85, 1.0, 0.75, 0.85, 0.85, 1.6, 0.8, 0.7, 0.7, 0.75, 0.65, 0.55, 0.65, 0.55, 0.45, 0.45, 0.45, 0.65, 0.5, 0.6, 0.55, 0.7, 0.7]

FAILURE
  - mean: 14.36
  - unbiased stdev: 4.73897
  - unbiased sterr: 0.94779
  - margin of error (95% conf. lvl.) = +/-1.858

RATE:
  - mean: 71.80%
  - unbiased stdev: 23.69487%
  - unbiased sterr: 4.73897%
  - margin of error (95% conf. lvl.) = +/-9.288%

### Round 21

Condition: Constant moderate pressure, constant high load

Parameters:
  - 2800 tps
  - 2800 total tx
  - 50 testing cycle
  - 50 rounds per cycle


