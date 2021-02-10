#!/bin/bash

TOOL=../target/release/automata
NODE=automata

secret=$1
COUNTER=4
i=1

echo "---------------------"
echo "generate stash controller key"
echo "---------------------"

while(( $i<=$COUNTER ))
do
  for j in stash controller ; do
      $TOOL key inspect-key //"$secret"//$NODE//$j//$i;
      done;

  # used for grandpa
  echo "------ ed25519 ------"
  $TOOL key inspect-key --scheme Ed25519 //"$secret"//$NODE//session//$i;
  # used for babe, ImOnline , AuthorityDiscovery
  echo "------ sr25519 ------"
  $TOOL key inspect-key //"$secret"//$NODE//session//$i;

  let "i++"
done

echo "---------------------"
echo "generate root key"
echo "---------------------"

$TOOL key inspect-key //"$secret"//$NODE;

echo "---------------------"
echo "generate witness key ecdsa"
echo "---------------------"

k=1
while(( $k<=3 ))
do
  # used for witness
  $TOOL key inspect-key --scheme Ecdsa //"$secret"//$NODE//witness//$k;

  let "k++"
done

echo "---------------------"
echo "generate relayer key ecdsa"
echo "---------------------"

$TOOL key inspect-key --scheme Ecdsa //"$secret"//$NODE//relayer;