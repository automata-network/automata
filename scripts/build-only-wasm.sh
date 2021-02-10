
# Script for building only the WASM binary of the given project.

set -e

PROJECT_ROOT=`git rev-parse --show-toplevel`

CRATE_NAME="automata-runtime"

export WASM_TARGET_DIRECTORY=$(pwd)

cargo build --release -p $CRATE_NAME
