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

## Run

Start a local testnet using latest [docker image](https://hub.docker.com/r/atactr/automata):

```bash
./scripts/run-network.sh
```

Start a local testnet using locally built image to evaluate local changes:

```bash
./scripts/run-network.sh --build
```

## License

[Apache 2.0](./LICENSE)

