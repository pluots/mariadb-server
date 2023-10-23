#!/bin/sh

set -eaux

echo "1;a7addd9adea9978fda19f21e6be987880e68ac92632ca052e5bb42b1a506939a" > /file-keys.txt

mariadbd --user=root \
    --datadir=/usr/local/mysql/data \
    --plugin-maturity=experimental \
    --log-bin=on \
    --loose-file-key-management-filename=/file-keys.txt \
    --loose-file-key-management-chacha-filename=/file-keys.txt
    # --encrypt-binlog=on \
    # --innodb-encrypt-log=on \
    # --plugin-load-add='file_key_management_chacha=encryption_chacha'