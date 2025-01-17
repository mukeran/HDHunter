#!/bin/sh

mkdir -p build/hdhunter/apache/lib/

# Copy dependencies
python3 $TARGETS_ROOT/tools/copy_dependencies.py build/hdhunter/apache/bin/httpd build/hdhunter/apache/lib/

# Copy config
cp config/httpd.conf build/hdhunter/apache/conf/

# Pack
tar -cvf $TARGETS_ROOT/dist/apache-resp.tar build
tar -rvf $TARGETS_ROOT/dist/apache-resp.tar -C startup setup.sh start.sh check_payload port mode
gzip -f $TARGETS_ROOT/dist/apache-resp.tar
