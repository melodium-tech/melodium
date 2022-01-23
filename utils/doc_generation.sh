#!/usr/bin/env bash

OUTPUT="$1"

mkdir -p "$OUTPUT"

IFS=$'\n'
for ITEM in `cargo run -- --doc-list --stdlib melodium-tests/convenience/empty_std melodium-tests/convenience/empty_main.mel`
do
    QUALIFICATION=`sed -E 's#^\(([A-Za-z]+)\).*#\1#' <<< $ITEM`
    PATH_PART=`sed -E 's#^\([A-Za-z]+\) ([a-z0-9/]+).*$#\1#' <<< $ITEM`
    NAME_PART=`sed -E 's#.*::([A-Za-z0-9]+)$#\1#' <<< $ITEM`
    
    echo $QUALIFICATION
    echo $PATH_PART
    echo $NAME_PART
    
    mkdir -p "$OUTPUT/$PATH_PART"
    
    cargo run -- --doc "$PATH_PART::$NAME_PART" --stdlib melodium-tests/convenience/empty_std melodium-tests/convenience/empty_main.mel > "$OUTPUT/$PATH_PART/$NAME_PART.part"
done
unset IFS

