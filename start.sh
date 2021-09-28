#!/bin/bash
      
      secret=//boolnetwork
      RUST_LOG=runtime=debug /home/liaoyi/projects/automata/target/release/automata key insert --scheme=ed25519 --chain=contextfree --suri=$secret//automata//session//1 --key-type=gran \
          --base-path=/tmp/bootnode1 --keystore-path=/tmp/bootnode1/keystore
      
      for key in babe imon audi; do
        RUST_LOG=runtime=debug /home/liaoyi/projects/automata/target/release/automata key insert --chain=contextfree --suri=$secret//automata//session//1 --key-type=$key \
          --base-path=/tmp/bootnode1 --keystore-path=/tmp/bootnode1/keystore
      done
