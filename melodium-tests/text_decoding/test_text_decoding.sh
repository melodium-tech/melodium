#!/usr/bin/env bash

set -e
FAILURES=0
declare -i FAILURES

function check_diff() {
    ENCODING="$1"
    FILE="$2"
    REFERENCE="$3"

    if diff "$REFERENCE" "$FILE" >> /dev/null
    then
        echo "$ENCODING OK"
    else
        >&2 echo "$ENCODING FAILED"
        FAILURES+=1
    fi
}

"$MELODIUM" reencoding.mel

check_diff utf-8 output_for_input_utf-8.txt reference_utf-8.txt
check_diff iso-8859-15 output_for_input_iso-8859-15.txt reference_iso-8859-15.txt
check_diff ascii output_for_input_ascii.txt reference_ascii.txt
check_diff utf-16le output_for_input_utf-16le.txt reference_utf-16le.txt
check_diff utf-16be output_for_input_utf-16be.txt reference_utf-16be.txt

exit $FAILURES
