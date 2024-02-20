#!/bin/bash

trap 'kill $(jobs -pr)' EXIT SIGINT SIGTERM
jackd -P90 -t2000 -dalsa -dhw:USB -r96000 -p256 -n3 &
sleep 2

FNAME="${HOME}/.jack-patchbay"
[[ -n $1 ]] && FNAME=$1
jack-patcher $FNAME
