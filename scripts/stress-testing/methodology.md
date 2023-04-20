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
- One iteration will consist of the test repeated 20 times with the same parameters.
- After an iteration both parameters will be increased by 256 until a significant
  failure is detected.

A significant failure includes:
 - an crashed node runtime (manifesting as a 6 second blocktime) in 2 or more tests per iteration
 - TODO

## Phase 2

## Phase 3
