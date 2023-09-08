#!/bin/sh
# Start a built MDB instance with user `mysql` 

set -eaux

# Setup for file plugin examples
# install plugin file_key_management soname 'file_key_management.so';
# https://mariadb.com/kb/en/file-key-management-encryption-plugin/
echo "1;a7addd9adea9978fda19f21e6be987880e68ac92632ca052e5bb42b1a506939a" > /file-keys.txt

/usr/local/mysql/bin/mariadbd-safe --user=mysql \
    --plugin-maturity=experimental \
    --log-error=/error.log \
    --general-log=on \
    --general-log-file=/general.log &
    # --log-bin=on \
    # --encrypt-binlog=on \
    # --innodb-encrypt-log=on \
    # --plugin-load-add='file_key_management_chacha=encryption_chacha' \
    # --loose-file-key-management-filename=/file-keys.txt \
    # --loose-file-key-management-chacha-filename=/file-keys.txt &
tail -f /error.log
