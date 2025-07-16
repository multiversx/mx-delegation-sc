# Change Log

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [0.5.9] - 2025-07-16
- Governance `delegateVote` & `getVotingPower`, compiled from the old `v0.5.8` tag, using elrond-wasm 0.23.0.

## [0.5.8] - 2021-11-24
- upgraded to elrond-wasm 0.23.0
- `unStakeTokens`/`unBondTokens` endpoints

## [0.5.7] - 2021-06-23
- `dnsRegister` via the DNS module
- `forceUnStakeNodesCallback` - which corrects a callback that ran out of gas on the mainnet
- Dust cleanup functionality, to get rid of small stakes.

## [0.5.6] - 2021-04-27
- call unBondNodes in the protocol, which unbonds the nodes without the tokens

## [0.5.5] - 2021-04-11
- unbond doesn't run out of gas, it simply interrupts its execution

## [0.5.4] - 2021-03-20
- Ability to unstake nodes without unstaking the tokens
- setNumBlocksBeforeUnBond renamed (capital "B") for consistency

## [0.5.3] - 2021-01-14
- getFullActiveList fix
- fixed missing user mappings from genesis

## [0.5.2] - 2021-01-13
- getFullActiveList implementation
- bls signature 48 bytes
- elrond-wasm 0.9.7
- bytecode size improvement

## [0.5.1] - 2020-10-14
- lift unBondNodes block nonce restriction

## [0.5.0]
- Introduced the delegation cap and the reset checkpoint system for adjusting it in more than one tx.

## [0.4.1]
- Fixed settings after genesis.

## [0.4.0]
- Genesis version of the contract. Only contains genesis initialization and readonly endpoints.

