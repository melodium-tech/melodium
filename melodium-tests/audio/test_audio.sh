#!/usr/bin/env bash

set -e

"$MELODIUM" --stdlib /tmp/fake_std audio_wave.mel

sha256sum reencoded_*
echo hexdump -C ./reencoded_i16.wav
hexdump -C ./reencoded_i16.wav

echo hexdump -C ./reencoded_i24.wav
hexdump -C ./reencoded_i24.wav

echo hexdump -C ./reencoded_f32.wav
hexdump -C ./reencoded_f32.wav

sha256sum -c reencoded.sha256sums
