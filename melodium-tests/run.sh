#!/usr/bin/env bash

ORIGINAL_DIR="$PWD"
cd `dirname $BASH_SOURCE`

if [ -z "$MELODIUM" ]
then
    MELODIUM='melodium-rust'
fi

export PATH="`dirname $BASH_SOURCE`/../target/debug:$PATH"
export MELODIUM="$MELODIUM"
export RUST_BACKTRACE=1

function run_test() {
    TEST_NAME="$1"
    
    date +"%Y-%m-%d %T"
    echo "Running $TEST_NAME…"
    
    cd "$TEST_NAME"
    
    "./test_$TEST_NAME.sh"
    RESULT=$?
    
    cd ..
    
    if [ "$RESULT" -eq 0 ]
    then
        echo "$TEST_NAME OK"
    else
        echo "$TEST_NAME FAILURE ($RESULT)"
    fi
}


mkdir -p /tmp/fake_std
rm -rf /tmp/tests
mkdir -p /tmp/tests

echo Running Mélodium tests…

# Add tests there
run_test generation_conversion_bytes

date +"%Y-%m-%d %T"
echo Run finished

cd "$ORIGINAL_DIR"
