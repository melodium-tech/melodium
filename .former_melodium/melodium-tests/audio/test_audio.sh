#!/usr/bin/env bash

set -e

rm -f reencoded_*.wav

"$MELODIUM" audio_wave.mel

sha256sum -c reencoded.sha256sums
