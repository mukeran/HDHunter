#!/bin/sh
mkdir /tmp/apache
mkdir /tmp/apache/conf
mkdir /tmp/apache/htdocs
mkdir /tmp/apache/logs
cp -r ../build/hdhunter/apache/conf/* /tmp/apache/conf
cp -r ../build/hdhunter/apache/htdocs/* /tmp/apache/htdocs
# ipcmk -M 65536
# ../../../tools/monitor_http_param create
export __AFL_SHM=196608 __AFL_SHM_SIZE=65536 __HTTP_PARAM=196609 __HTTP_PARAM_SIZE=288 __EXECUTION_PATH=196610 __EXECUTION_PATH_SIZE=8
export LD_LIBRARY_PATH=$(realpath ../build/hdhunter/apache/lib)
# export HDHUNTER_TRACE=/tmp/hdhunter_trace_apache
$(realpath ../build/hdhunter/apache/bin/httpd) -X -f $(realpath ../config/debug.conf) -DFOREGROUND
# nohup $(realpath ../build/hdhunter/apache/bin/httpd) -X -f $(realpath ../config/debug.conf) &
# while ! curl http://127.0.0.1:2080; do
#     sleep 0.1
# done
