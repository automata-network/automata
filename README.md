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

Docker-compose files is providered for settting up an local/staging testnet with 2 validator nodes and 1 lightnode. 

Names of docker-compose file that ends with `-build.yml` build automata node locally, others pull images from DockerHub.

```
docker-compose-local.yml
docker-compose-staging.yml
docker-compose-local-build.yml
docker-compose-staging-build.yml
```
## License

[Apache 2.0](./LICENSE)

