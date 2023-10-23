#!/bin/sh

# Launch without going through cmake install

set -eaux

mariadb-install-db

cp /checkout/rust/scripts/start_maria.sh /usr/local/bin/start-mdb
cp /checkout/rust/scripts/my.cnf ~/my.cnf

cat << EOF > /usr/local/bin/kill-mdb
#!/bin/sh

pkill mariadbd
EOF

cat << EOF > /etc/mysql/conf.d/custom.cnf
[mariadb]
log_warnings=9
EOF

chmod +x /usr/local/bin/start-mdb
chmod +x /usr/local/bin/kill-mdb
mkdir -p /run/mysqld/

/checkout/rust/scripts/copy_plugins.sh

start-mdb
