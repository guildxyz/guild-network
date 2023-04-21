# Methodology of testing

The goal of the testing is to determine the minimum stress level at which the
probability of failure is less than 5% and greater than 95%, respectively. These
values will provide reliable metrics of the stress levels at which the chain
will no longer function reliably. The testing will involve the following phases:

1. Phase 1: Determine the initial failure point of the chain by subjecting it to
   progressively increasing stress levels until a significant failure is
   observed.
2. Phase 2: Establish a probability distribution of failure by continuing to
   increase the stress levels in smaller increments and recording the number of
   times the chain fails.
3. Phase 3: Determine the lowest stress levels at which the chain reliably fails
   with a probability of 5% and 95%, respectively.

## Definitions

- Force: the total amount of extrinsics sent,
- Pressure: the extrinsics sent per second (EPS).

## Phase 1

In order to identify the minimum parameters that result in a failure probability
greater than 5%, we will utilize the following approach:

- Testing will begin by both parameters set to 256.
- One test cycle will consist of 20 iterations of the same test repeated with
  the same parameters.
- After a test cycle, both parameters will be increased by 256 until a
  significant failure is detected.

A significant failure is defined as:
 - a crashed node runtime (manifesting as a 6 second blocktime) in more than 20%
   of the iterations
 - less extrinsics being overall executed than sent initially in a test
 - a statistically significant (3 sigma) amount of anomalies manifesting in more
   than 20% of the iterations, consisting of minor block time anomalies, minor
   deviations in expected block size, and minor deviations in both mean and
   dispersion

## Phase 2

## Phase 3
