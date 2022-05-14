#!/usr/bin/env bash

set -e
FAILURES=0
declare -i FAILURES

rm -f output*

"$MELODIUM" functions.mel

test $(cat output.txt) == "42"

exit $FAILURES
