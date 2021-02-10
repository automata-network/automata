[![Rust](../../workflows/Rust/badge.svg)](../../actions?query=workflow%3ARust)
# Automata

## Build

On Ubuntu/Debian, install the following packages:

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

## License

[Apache 2.0](./LICENSE)


