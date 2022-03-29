#!/usr/bin/env bash

set -e

"$MELODIUM" --stdlib /tmp/fake_std audio_wave.mel

sha256sum reencoded_*
head reencoded_*
sha256sum -c reencoded.sha256sums
