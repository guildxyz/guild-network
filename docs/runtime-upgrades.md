# Runtime upgrade history

## 2023-04-19
Runtime version bumped from `103` to `104`. [Respective
PR](https://github.com/agoraxyz/guild-network/pull/138)

### Detailed info
This [release](https://github.com/agoraxyz/guild-network/releases/tag/alpha-runtime-104)
enabled the runtime to accept extrinsics that were signed by EVM-specific wallets
(e.g. Metamask). This was simply achieved by replacing the runtime's `MultiSignature`
type to our own implementation that is compatible with EVM-specific ECDSA signatures.
After the frontend is updated accordingly, users can interact with the guild pallet via
Metamask. `Sr25519` and `Ed25519` signatures are still supported without any change.

### Required steps for node operators
#### Validator nodes
- no steps required
#### Oracle nodes
- no steps required

## 2023-03-30
Runtime version bumped from `102` to `103`. [Respective
PR](https://github.com/agoraxyz/guild-network/pull/123)

### Detailed info
This [release](https://github.com/agoraxyz/guild-network/releases/tag/alpha-runtime-103)
simply changed the `pallet_im_online::Config::ReportUnresponsiveness` type from
`()` to `ValidatorManager`. This means that the `ValidatorManager` pallet is
now responsible for automatically removing unresponsive (offline) validators
after approximately two sessions. Validators consistently sending heartbeats
are safe.

In case a validator gets removed from the active validator set due to going
offline, they can still join the active validator set once they come online
again; the operator just needs to submit a `ValidatorManager >
addValidatorAgain` transaction (from the explorer).

### Required steps for node operators
#### Validator nodes
- no steps required
#### Oracle nodes
- no steps required

## 2023-03-22
Runtime version bumped from `101` to `102`. [Respective
PR](https://github.com/agoraxyz/guild-network/pull/114)

### Detailed info
This [release](https://github.com/agoraxyz/guild-network/releases/tag/alpha-runtime-102)
extended the runtime with `pallet-im-online`. This pallet is responsible for
monitoring heartbeats of online validator nodes and it automatically removes
validators from `pallet-validator-manager`'s `Validators` vector when they go
offline.

Adding a fully functional `pallet-im-online` integration to a live chain is actually a two-step
process and this upgrade is the first step. Since every validator needs an additional `im-online`
key in their `SessionKeys` this upgrade automatically assigned dummy keys to each validator. In order
to replace these dummy keys, each validator needs to rotate their keys again.

You can see in the
[explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F1.oracle.network.guild.xyz#/explorer)
that most validators are assumed to be offline because they are not sending
heartbeats to the pallet due to their dummy keys. Note that Guild Network nodes
have already rotated their keys and they are sending heartbeats.

The only reason nodes that are assumed to be offline are not removed automatically from the
active validator set is that the authority for removing them is set to `()` temporarily which
means that nothing happens if someone goes offline. However, the second step in
the `pallet-im-online` integration will change the authority from `()` to
`pallet-valiator-manager`. From that point on, if any validator goes offline
and fails to send heartbeats, it will be automatically removed from the active
validator set.

Don't worry if your node goes offline and gets removed from the active
validator set, you can rotate your keys again. You'll receive 96 bytes of keys
in the response which you should split into three 32 byte long keys:

- aura (32 bytes, add `0x` to the front)
- grandpa (32 bytes, add `0x` to the front)
- im-online (32 bytes, add `0x` to the front)

and submit them in the explorer via `Session > setKeys`. You should simply add
`0x` as the proof. Then, you should submit the `ValidatorManager >
addValidatorAgain` transaction from the explorer
to re-join the active validator set.

### Required steps for node operators
#### Validator nodes
- rotate your session keys as soon as possible (you need unsafe rpc methods available)
#### Oracle nodes
- no steps required

## 2023-03-20
Runtime version bumped from `100` to `101`. [Respective
PR](https://github.com/agoraxyz/guild-network/pull/117)

### Detailed info
This [release](https://github.com/agoraxyz/guild-network/releases/tag/alpha-runtime-101)
fixed a bug in `pallet-validator-manager`. This pallet has (among others) two
main storages: `Validators` and `ApprovedValidators`. `Validators` is a subset
of `ApprovedValidators` containing all validators that should be actively
validating in the network. `ApprovedValidators` contains all validators that
has ever been approved (added) by someone with sudo access. Thus, if someone
stops validating, they may be removed from the `Validators` storage but their
account remains in `ApprovedValidators`. Thus, they can call
`add_validator_again` if they wish to start actively validating again. They can
only do this if they are still members of `ApprovedValidators`.

When a new validator is added via the pallet using the sudo account, the
validator's address is saved in both `Validators` and `ApprovedValidators`.
However, due to a bug when the `remove_validator` call was executed by the sudo
account, it only removed the given validator from the `Validators` storage, but
not the `ApprovedValidators` storage (the respective storage write was not
saved). Thus, the sudo account couldn't add the removed validator again because
it was a member in `ApprovedValidators`.

### Required steps for node operators
#### Validator nodes
- no steps required
#### Oracle nodes
- no steps required
