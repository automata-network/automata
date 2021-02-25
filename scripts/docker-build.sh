#!/bin/bash

if ! which docker >/dev/null 2>&1
then
    echo "Please install docker first"
    exit 1
fi

if [ ! -f "${PWD}/Dockerfile" ]
then
    echo "Please run this script from the project's root"
    exit 1
fi

## Docker buildx cache location
##
## The default cache location is stored within cargo target folder, which will
## be removed upon `cargo clean`. Change it to other value if you need more
## persistent cache.
##
## This can be set through environment variable DOCKER_CACHE_DIR
CACHE_DIR="${DOCKER_CACHE_DIR:-${PWD}/target/docker-cache}"

## Image tag
##
## The default tag is automata
##
## This can be set throught environment variable DOCKER_TAG
TAG="${DOCKER_TAG:-automata}"

## Use cache from docker registry, if local cache is not found
if [ -d "$CACHE_DIR" ]
then
    CACHE_FROM="type=local,src=${CACHE_DIR}"
else
    echo "Warning: Local docker build cache not found at ${CACHE_DIR}"
    echo "Pulling docker image for remote cache"
    CACHE_FROM="type=registry,ref=atactr/automata:latest"
fi

if docker buildx >/dev/null 2>&1
then
    ## Create automata-docker-builder instance for buildx
    docker buildx inspect automata-docker-builder >/dev/null 2>&1 || docker buildx create --driver docker-container --name automata-docker-builder --use
    ## Build with buildx to allow cross-build cache
    docker buildx build --cache-from="${CACHE_FROM}" --cache-to="type=local,mode=max,dest=${CACHE_DIR}" --tag "${TAG}" --load .
else
    ## No docker buildx plugin found, fallback to default docker build command without any cache
    ##
    ## If you need faster build next time, install docker buildx: https://github.com/docker/buildx#installing
    DOCKER_BUILDKIT=1 docker build --tag "${TAG}" .
fi


