#!/usr/bin/env bash

# This script is a _temporary_ way to generate core reference for Mélodium, it should be removed when the main executable will be able to generate full documentation by itself.

OUTPUT="$1"

if [ -z "$MELODIUM" ]
then
    MELODIUM='target/debug/melodium-rust'
fi

mkdir -p "$OUTPUT"

IFS=$'\n'
for ITEM in `$MELODIUM --doc-list --stdlib melodium-tests/convenience/empty_std melodium-tests/convenience/empty_main.mel`
do
    QUALIFICATION=`sed -E 's#^\(([A-Za-z]+)\).*#\1#' <<< $ITEM`
    PATH_PART=`sed -E 's#^\([A-Za-z]+\) ([a-z0-9/]+).*$#\1#' <<< $ITEM`
    NAME_PART=`sed -E 's#.*::([A-Za-z0-9]+)$#\1#' <<< $ITEM`
    
    mkdir -p "$OUTPUT/$PATH_PART"
    
    "$MELODIUM" --doc "$PATH_PART::$NAME_PART" --stdlib melodium-tests/convenience/empty_std melodium-tests/convenience/empty_main.mel > "$OUTPUT/$PATH_PART/$NAME_PART"
done

echo '[book]
language = "en"
multilingual = false
src = "core"
title = "Mélodium Core Reference"' > $OUTPUT/book.toml

echo '# Reference' > $OUTPUT/core/SUMMARY.md
echo '[Core](main.md)' >> $OUTPUT/core/SUMMARY.md

echo "# Core Reference

> Mélodium and this reference are a work in progress, aiming to evolve quickly and significantly with time.
> All the informations explained there might no be up-to-date compared to the current state of the project.
> All this work is done with passion and any comment is good to provide.
" > $OUTPUT/core/main.md

for LOCATION in `find "$OUTPUT/core" -type d -not -wholename "$OUTPUT/core" | sort`
do
    COMPLETE_NAME=`sed s#$OUTPUT/core/## <<< $LOCATION`
    NAME=`basename $COMPLETE_NAME`
    
    echo "[$COMPLETE_NAME](./$COMPLETE_NAME/main.md)" >> $OUTPUT/core/SUMMARY.md
    
    cd $LOCATION
    
    echo "# Area \`$NAME\`" > main.md
    echo "\`core/$COMPLETE_NAME\`" >> main.md
    
    SUBMODS=`find . -mindepth 1 -maxdepth 1 -type d`
    
    if [ -n "$SUBMODS" ]
    then
        echo "## Subareas" >> main.md
        
        for SUBMOD in `find . -mindepth 1 -maxdepth 1 -type d | sort`
        do
            NAME=`sed s#./## <<< $SUBMOD`
            echo "- [$NAME]($SUBMOD/main.md)" >> main.md
        done
    fi
    
    ITEMS=`find . -maxdepth 1 -type f -not -name '*.md'`
    
    if [ -n "$ITEMS" ]
    then
        echo "## Elements" >> main.md
        
        for ITEM in `find . -maxdepth 1 -type f -not -name '*.md' | sort`
        do
            NAME=`sed s#./## <<< $ITEM`

            echo "### $NAME" >> main.md
            cat $ITEM >> main.md
        done
    fi
    
done

unset IFS

