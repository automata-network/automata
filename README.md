[![Rust](../../workflows/Rust/badge.svg)](../../actions?query=workflow%3ARust+branch%3Amain) [![Docker](../../workflows/Docker/badge.svg)](../../actions?query=workflow%3ADocker+branch%3Amain)
# Automata

## Build

On Ubuntu/Debian (or similar distributions on [WSL](https://docs.microsoft.com/en-us/windows/wsl/about)), install the following packages:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config llvm-dev libclang-dev clang libssl-dev curl
```

Install Rust through [rustup.rs](https://rustup.rs):

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

Initialize your Wasm Build environment:

```
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release --features finitestate
```

## Run
### Local Two-node Testnet
If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet. You'll need two terminals open. In one, run:
```bash
./target/release/automata --chain=local --alice -d /tmp/alice
```
And in the other, run:
```bash
./target/release/automata --chain=local --bob -d /tmp/bob --port 30334 --bootnodes '/ip4/127.0.0.1/tcp/30333/p2p/<ALICE_BOOTNODE_ID_HERE>'
Ensure you replace `ALICE_BOOTNODE_ID_HERE` with the node ID from the output of the first terminal.
```

You can muck around by heading to https://polkadot.js.org/apps and choose "Local Node" from the Settings menu. Make sure you have the following type definitions saved in the Settings->Developer page:
```json
{
  "ResourceId": "[u8; 32]",
  "DepositNonce": "u64",
  "ProposalVotes": {
    "votes_for": "Vec<AccountId>",
    "votes_against": "Vec<AccountId>",
    "status": {
      "_enum": [
        "Initiated",
        "Approved",
        "Rejected"
      ]
    }
  },
  "BridgeTokenId": "U256",
  "BridgeChainId": "u8",
  "VestingPlan": {
    "start_time": "u64",
    "cliff_duration": "u64",
    "total_duration": "u64",
    "interval": "u64",
    "initial_amount": "Balance",
    "total_amount": "Balance",
    "pub vesting_during_cliff": "bool"
  }
}
```

## License

[Apache 2.0](./LICENSE)
