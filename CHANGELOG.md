# Change Log

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [0.5.8]
- upgraded to elrond-wasm 0.23.0
- `unStakeTokens`/`unBondTokens` endpoints

## [0.5.7]
- `dnsRegister` via the DNS module
- `forceUnStakeNodesCallback` - which corrects a callback that ran out of gas on the mainnet
- Dust cleanup functionality, to get rid of small stakes.

## [0.5.6]
- call unBondNodes in the protocol, which unbonds the nodes without the tokens

## [0.5.5]
- unbond doesn't run out of gas, it simply interrupts its execution

## [0.5.4]
- Ability to unstake nodes without unstaking the tokens
- setNumBlocksBeforeUnBond renamed (capital "B") for consistency

## [0.5.3]
- getFullActiveList fix
- fixed missing user mappings from genesis

## [0.5.2]
- getFullActiveList implementation
- bls signature 48 bytes
- elrond-wasm 0.9.7
- bytecode size improvement

## [0.5.1]
- lift unBondNodes block nonce restriction

## [0.5.0]
- Introduced the delegation cap and the reset checkpoint system for adjusting it in more than one tx.

## [0.4.1]
- Fixed settings after genesis.

## [0.4.0]
- Genesis version of the contract. Only contains genesis initialization and readonly endpoints.

