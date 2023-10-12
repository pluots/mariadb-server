# Launch without going through cmake install

set -eaux

echo "1;a7addd9adea9978fda19f21e6be987880e68ac92632ca052e5bb42b1a506939a" > /file-keys.txt

mariadb-install-db

cat << EOF > /usr/local/bin/start-mdb
#!/bin/sh

mariadbd --user=root \
    --plugin-maturity=experimental \
    --log-bin=on \
    --loose-file-key-management-filename=/file-keys.txt \
    --loose-file-key-management-chacha-filename=/file-keys.txt
    # --encrypt-binlog=on \
    # --innodb-encrypt-log=on \
    # --plugin-load-add='file_key_management_chacha=encryption_chacha'
EOF

cat << EOF > /usr/local/bin/kill-mdb
#!/bin/sh

pkill mariadbd
EOF

cat << EOF > /etc/mysql/conf.d/custom.cnf
[mariadb]
log-warnings=9
EOF

chmod +x /usr/local/bin/start-mdb
chmod +x /usr/local/bin/kill-mdb
mkdir -p /run/mysqld/

/checkout/rust/scripts/copy_plugins.sh

start-mdb
