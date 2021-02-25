#!/bin/bash

## Usage: see the output of '--help' option

set -eo pipefail

if ! which docker-compose >/dev/null 2>&1
then
    echo "Please install docker-compose first: https://docs.docker.com/compose/install/"
    exit 1
fi

if [ ! -f "${PWD}/docker-compose.yml" ]
then
    echo "Please run this script from the project's root"
    exit 1
fi

## Default arguments
##
## use local image built by `scripts/docker-build.sh`
IMAGE=automata
## use 'local' chainspec
CHAIN=local 
## build a new image using current version before running the network
BUILD=0
## remove containers created by the docker-compose file
RM=0

while [[ $# -gt 0 ]]
do
    arg_name="$1"
    case ${arg_name} in
        --chain)
            CHAIN="$2"
            shift; shift
        ;;
        --image)
            IMAGE="$2"
            shift; shift
        ;;
        --build)
            BUILD=1
            shift
        ;;
        --rm)
            RM=1
            shift;
        ;;
        -h|--help)
            cat <<EOF
Usage: $0 [--chain] [--image]

Options:
    --chain CHAIN
        Select one of the supported chain spec.

    --image IMAGE
        Set the docker image. Defaults to 'atactr/automata'

    --build
        Build image using current version and run

    --help
        print this help message
		
	--rm
        remove containers created by the docker-compose file
EOF
        exit 1
        ;;
    esac
done 

# Argument validation for --chain
case ${CHAIN} in
    local|staging)
    ;;
    *)
    echo "Invalid argument '$CHAIN' for --chain. Available ones are 'local' and 'staging'"
esac

if [ "$BUILD" -eq 1 ]
then
    echo "Building the image 'automata'"
    scripts/docker-build.sh
fi

echo "Running the network"
echo "Image: $IMAGE"
echo "Chain: $CHAIN"

# Export environment variables for docker-compose.yml
export CHAIN
export IMAGE

if [ "$RM" -eq 1 ]
then
    echo "Remove containers created by the docker-compose file"
    docker-compose rm
else
    docker-compose up
fi