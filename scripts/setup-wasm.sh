#!/bin/sh

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


BRANCH_NAME=$(echo $GITHUB_REF | cut -d'/' -f 3)

docker run --rm -i \
  -e PACKAGE=automata-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -v ${WORK_PATH}:/build \
  chevdor/srtool:${SR_TOOL_TAG} build \
  | tee ${WORK_PATH}/build-automata-wasm.log


docker run --rm -i \
  -e PACKAGE=contextfree-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -v ${WORK_PATH}:/build \
  chevdor/srtool:${SR_TOOL_TAG} build \
  | tee ${WORK_PATH}/build-contextfree-wasm.log


mkdir -p ${WORK_PATH}/deploy/bin

_PROPOSAL_AUTOMATA=$(cat ${WORK_PATH}/build-automata-wasm.log | grep 'Proposal\s\+:')
_PROPOSAL_CONTEXTFREE=$(cat ${WORK_PATH}/build-contextfree-wasm.log | grep 'Proposal\s\+:')

PROPOSAL_AUTOMATA=0x${_PROPOSAL_AUTOMATA#*0x}
PROPOSAL_CONTEXTFREE=0x${_PROPOSAL_CONTEXTFREE#*0x}

PROPOSAL_AUTOMATA=$(echo ${PROPOSAL_AUTOMATA} | sed 's/[^[:print:]]\[0m//g')
PROPOSAL_CONTEXTFREE=$(echo ${PROPOSAL_CONTEXTFREE} | sed 's/[^[:print:]]\[0m//g')


echo ${PROPOSAL_AUTOMATA} > ${WORK_PATH}/deploy/bin/${PROPOSAL_AUTOMATA}.proposal.automata.txt
echo ${PROPOSAL_CONTEXTFREE} > ${WORK_PATH}/deploy/bin/${PROPOSAL_CONTEXTFREE}.proposal.contextfree.txt


## after srtool 2021-03-15 use this
cp ${WORK_PATH}/runtime/automata/target/srtool/release/wbuild/automata-runtime/automata_runtime.compact.wasm \
 ${WORK_PATH}/deploy/bin/
cp ${WORK_PATH}/runtime/contextfree/target/srtool/release/wbuild/contextfree-runtime/contextfree_runtime.compact.wasm \
 ${WORK_PATH}/deploy/bin/

ls ${WORK_PATH}/deploy/bin/