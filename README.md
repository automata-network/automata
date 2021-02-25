[![Rust](../../workflows/Rust/badge.svg)](../../actions?query=workflow%3ARust)
# Automata

## Build

On Ubuntu/Debian (or similar distributions on [WSL](https://docs.microsoft.com/en-us/windows/wsl/about)), install the following packages:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config llvm-dev libclang-dev clang libssl-dev curl
```

Install Rust through [rustup.rs](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Initialize your Wasm Build environment:

```
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```
## Docker-compose

Docker-compose files is providered for settting up a local testnet with 2 validator nodes and 1 lightnode. 

Start a local network from remote docker images:
```
./scripts/run-network.sh --chain local --image atactr/automata
```

Start a local network after building native code:
```
./scripts/run-network.sh --chain local --build
```

## License

[Apache 2.0](./LICENSE)

