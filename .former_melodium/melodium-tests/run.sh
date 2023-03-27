#!/usr/bin/env bash

ORIGINAL_DIR="$PWD"
TEST_DIR="$(dirname $(realpath $BASH_SOURCE))"
cd "$TEST_DIR"

if [ -z "$MELODIUM" ]
then
    MELODIUM='melodium'
fi

export PATH="$TEST_DIR/../target/debug:$PATH"
export MELODIUM="$MELODIUM"
export RUST_BACKTRACE=1

GLOBAL_RESULT=0
declare -i GLOBAL_RESULT
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
        GLOBAL_RESULT+=1
    fi
}

echo Running Mélodium tests…

# Add tests there
run_test functions
run_test engine_io
run_test text_decoding
run_test text_encoding
run_test audio
run_test generation_conversion_bytes

date +"%Y-%m-%d %T"
echo Run finished

cd "$ORIGINAL_DIR"

exit $GLOBAL_RESULT
