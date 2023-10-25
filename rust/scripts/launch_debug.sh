#!/usr/bin/sh

gdbserver :2345 mariadbd \
    --user=root \
    --datadir=/usr/local/mysql/data \
    --plugin-maturity=experimental \
    --log-bin=on \
    --loose-file-key-management-filename=/file-keys.txt \
    --loose-file-key-management-chacha-filename=/file-keys.txt \
    --gdb