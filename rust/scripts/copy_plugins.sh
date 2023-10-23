#!/bin/bash
# Copy plugins from the build directory to the plugin directory, if it exists

set -eauxo pipefail

export BUILD_DIR=/obj/build-mariadb

check1="/usr/lib/mysql/plugin/"
check2="/usr/local/mysql/lib/plugin/"
check3="/plugins/"

if [ -d "$check1" ]; then
    echo "copying to dir1"
    export PLUGIN_DIR="$check1"
elif [ -d "$check2" ]; then
    echo "copying to dir2"
    export PLUGIN_DIR="$check2"
elif [ -d "$check3" ]; then
    echo "copying to dir3"
    export PLUGIN_DIR="$check3"
else
    echo "directory not yet created, exiting"
    exit
fi


# Copy built plugins
debug_dir="$BUILD_DIR/rust_target/debug"
release_dir="$BUILD_DIR/rust_target/release"

if [ -d "$debug_dir" ]; then
    ls -d "$debug_dir"/* | grep '\.so$' | xargs -iINFILE cp INFILE "$PLUGIN_DIR"
fi

if [ -d "$release_dir" ]; then
    ls -d "$release_dir"/* | grep '\.so$' | xargs -iINFILE cp INFILE "$PLUGIN_DIR"
fi

for f in $(ls -d "$PLUGIN_DIR"* | grep -E '/lib\w*\.so$'); do
    mv "$f" $(echo "$f" | sed -E 's/\/lib(\w*\.so)$/\/\1/g')
done
