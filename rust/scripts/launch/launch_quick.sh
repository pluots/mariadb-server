# Launch without going through cmake install

set -eaux

echo "1;a7addd9adea9978fda19f21e6be987880e68ac92632ca052e5bb42b1a506939a" > /file-keys.txt

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

chmod +x /usr/local/bin/start-mdb
chmod +x /usr/local/bin/kill-mdb
mkdir /run/mysqld/

export BUILD_DIR=/obj/build-mariadb

# Copy built plugins
ls -d "$BUILD_DIR/rust_target/debug"/* | grep '\.so$' | xargs -iINFILE cp INFILE /usr/lib/mysql/plugin/
# relfiles=$(ls -d "$BUILD_DIR/rust_target/release"* || true | grep '\.so$')

for f in $(ls -d /usr/lib/mysql/plugin/* | grep -E '/lib\w*\.so$'); do
    mv "$f" $(echo "$f" | sed -E 's/\/lib(\w*\.so)$/\/\1/g')
done
