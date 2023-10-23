#!/bin/sh
# Install built MDB under user `mysql`
# https://mariadb.com/kb/en/generic-build-instructions/

set -eaux

mkdir -p /run/mysqld/
mkdir /data
mkdir /plugins
touch /error.log /general.log

"$BUILD_DIR/scripts/mariadb-install-db" \
    --user=root \
    --srcdir=/checkout \
    "--builddir=$BUILD_DIR" \
    --datadir=/data

ln -s "$BUILD_DIR/sql/mariadbd" "/usr/bin/mariadbd"

cp /checkout/rust/scripts/my.cnf ~/.my.cnf

/checkout/rust/scripts/copy_plugins.sh

# Steps for full install
# cmake --install "$BUILD_DIR"
# useradd mysql
# mkdir -p /usr/local/mysql/data
# chown -R mysql /usr/local/mysql/ /error.log /general.log
# chown -R mysql /usr/local/mysql/ /error.log /general.log
# /usr/local/mysql/scripts/mariadb-install-db \
#     --user=mysql \
#     --datadir=/usr/local/mysql/data
