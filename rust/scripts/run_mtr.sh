#!/bin/bash

set -eaux

echo running tests

/obj/build-mariadb/mysql-test/mtr \
    --force \
    --max-test-fail=40 \
    "--parallel=$(nproc)" \
    --suite=plugins-,"${@:2}" # only test plugins, re-pass given arguments

# --mem \
# export MTR_BINDIR=/obj/build-mariadb
# mkdir /test
# cd /test

# --suite=unit
