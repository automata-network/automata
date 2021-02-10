#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

toolchain=$(<rust-toolchain)

if [ -z "${toolchain}" ]
then
   "rust-toolchain file not set"
fi

if [ -z "$CI_PROJECT_NAME" ] ; then
   rustup toolchain install "${toolchain}"
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain "${toolchain}"