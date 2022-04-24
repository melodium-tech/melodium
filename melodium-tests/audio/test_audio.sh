#!/usr/bin/env bash

set -e

"$MELODIUM" audio_wave.mel

sha256sum -c reencoded.sha256sums
