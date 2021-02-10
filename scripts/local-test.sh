#!/bin/bash

set -e

clear() {
  rm -r alice bob
  rm -r alice.txt bob.txt
}

stop() {
  pkill automata
}

if [ $# -eq 1 ]; then
  command="$1"
  case "${command}" in
    clear)
      clear
      ;;
    stop)
      stop
      ;;
  esac
  exit 1
fi

RUST_LOG="info" ./automata --base-path "$PWD/alice" \
          --chain local \
          --alice       \
          --port 30333  \
          --ws-port 9945  \
          --rpc-port 9933 \
          --node-key 7777777777777777777777777777777777777777777777777777777777777771 \
          --validator >> alice.txt 2>&1 &

RUST_LOG="info" ./automata --base-path "$PWD/bob" \
          --chain local \
          --bob       \
          --port 30334  \
          --ws-port 9946  \
          --rpc-port 9933 \
          --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWPULbKmnp8qWn2TiP27X7HMijV2AjgfAqErR3kdDMq5i7 \
          --validator >> bob.txt 2>&1 &


# wait for running
sleep 2
