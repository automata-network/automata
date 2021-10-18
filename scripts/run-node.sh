#!/bin/bash

set -eo pipefail

NODE=full
CHAIN=contextfree
NAME=

while [[ $# -gt 0 ]]
do
    arg_name="$1"
    case ${arg_name} in
        --node-type)
            NODE="$2"
            shift; shift
        ;;
        --chain)
            CHAIN="$2"
            shift; shift
        ;;
        --name)
            NAME="$2"
            shift; shift
        ;;
        -h|--help)
            cat <<EOF
Usage: $0 [--node-type] [--chain] [--name]

Options:
    --node-type TYPE
        Select one of the supported chain node {validator, archive, full}.

    --chain CHAIN
        Select one of the supported chain {contextfree, automata, finitestate}.

    --name NAME
        The human-readable name for this node. A random name will be generated if not indicated.

    --help
        print this help message
        
EOF
        exit 1
        ;;
    esac
done

case ${CHAIN} in
    contextfree|automata|finitestate)
    ;;
    *|"")
        echo "Invalid argument for --chain. Available ones are 'contextfree', 'automata', or 'finitestate'"
        exit 1
    ;;
esac

CMD=

case ${NODE} in
    validator)
        CMD="/usr/local/bin/automata --chain=$CHAIN --port=30333 --base-path=/data --validator --rpc-methods=Unsafe"
    ;;
    archive)
        CMD="/usr/local/bin/automata --chain=$CHAIN --port=30333 --base-path=/data --rpc-cors=all --rpc-external --ws-external --pruning archive"
    ;;
    full)
        CMD="/usr/local/bin/automata --chain=$CHAIN --port=30333 --base-path=/data --rpc-cors=all --rpc-external --ws-external"
    ;;
    *|"")
        echo "Invalid argument for --node-type. Available ones are 'validator', 'archive' or 'full'"
        exit 1
esac

if [ -n "$NAME" ]; then
    CMD="${CMD} --name ${NAME}"
fi

${CMD}
