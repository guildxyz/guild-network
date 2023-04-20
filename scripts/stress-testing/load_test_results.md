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

