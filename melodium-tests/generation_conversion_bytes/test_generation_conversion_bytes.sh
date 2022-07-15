#!/usr/bin/env bash

set -e
FAILURES=0
declare -i FAILURES

function bash_generation() {
    CHECK_VALUE="$1"
    OUTPUT_FILE="$2"
    
    i=0
    while [ "$i" -lt 1024 ]
    do

        echo "$CHECK_VALUE" | xxd -r -p - 
        i=$((i+1)) 
    done >> "$OUTPUT_FILE"
}

function test_generation() {
    TEST_NAME="$1"
    MEL_SCRIPT="$2"
    CHECK_VALUE="$3"
    OUTPUT_BASE_FILE="$4"
    
    "$MELODIUM" "$MEL_SCRIPT"
    # Enable this line to generate values and ignore already present files
    bash_generation "$CHECK_VALUE" "$OUTPUT_BASE_FILE.bash_generation"

    
    if diff "$OUTPUT_BASE_FILE.bash_generation" "$OUTPUT_BASE_FILE.mel_generation" >> /dev/null
    then
        echo "$TEST_NAME OK"
    else
        >&2 echo "$TEST_NAME FAILED"
        FAILURES+=1
    fi
}

rm -f *.mel_generation *.bash_generation

test_generation "generate_and_convert_u8_to_bytes" "generate_and_convert_u8_to_bytes.mel" "2A" "generated_u8"
test_generation "generate_and_convert_u16_to_bytes" "generate_and_convert_u16_to_bytes.mel" "00 2A" "generated_u16"
test_generation "generate_and_convert_u32_to_bytes" "generate_and_convert_u32_to_bytes.mel" "00 00 00 2A" "generated_u32"
test_generation "generate_and_convert_u64_to_bytes" "generate_and_convert_u64_to_bytes.mel" "00 00 00 00 00 00 00 2A" "generated_u64"
test_generation "generate_and_convert_u128_to_bytes" "generate_and_convert_u128_to_bytes.mel" "00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 2A" "generated_u128"
test_generation "generate_and_convert_i8_to_bytes" "generate_and_convert_i8_to_bytes.mel" "D6" "generated_i8"
test_generation "generate_and_convert_i16_to_bytes" "generate_and_convert_i16_to_bytes.mel" "FF D6" "generated_i16"
test_generation "generate_and_convert_i32_to_bytes" "generate_and_convert_i32_to_bytes.mel" "FF FF FF D6" "generated_i32"
test_generation "generate_and_convert_i64_to_bytes" "generate_and_convert_i64_to_bytes.mel" "FF FF FF FF FF FF FF D6" "generated_i64"
test_generation "generate_and_convert_i128_to_bytes" "generate_and_convert_i128_to_bytes.mel" "FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF D6" "generated_i128"
test_generation "generate_and_convert_f32_to_bytes" "generate_and_convert_f32_to_bytes.mel" "C4 9A 52 2C" "generated_f32"
test_generation "generate_and_convert_f64_to_bytes" "generate_and_convert_f64_to_bytes.mel" "C0 93 4A 45 85 47 A0 9E" "generated_f64"
test_generation "generate_and_convert_bool_to_bytes" "generate_and_convert_bool_to_bytes.mel" "01" "generated_bool"
test_generation "generate_and_convert_char_to_bytes" "generate_and_convert_char_to_bytes.mel" "2A" "generated_char"
test_generation "generate_and_convert_string_to_bytes" "generate_and_convert_string_to_bytes.mel" "4D C3 A9 6C 6F 64 69 75 6D" "generated_string"


exit $FAILURES

