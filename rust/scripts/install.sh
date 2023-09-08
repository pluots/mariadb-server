#!/bin/sh
# Install built MDB under user `mysql`
# https://mariadb.com/kb/en/generic-build-instructions/

set -eaux

cmake --install "$BUILD_DIR"

useradd mysql

touch /error.log /general.log
mkdir -p /usr/local/mysql/data
chown -R mysql /usr/local/mysql/ /error.log /general.log

/usr/local/mysql/scripts/mariadb-install-db --user=mysql
