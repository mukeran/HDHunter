#!/bin/sh
mkdir /tmp/apache
mkdir /tmp/apache/conf
mkdir /tmp/apache/htdocs
mkdir /tmp/apache/logs
cp -r ../build/hdhunter/apache/conf/* /tmp/apache/conf
cp -r ../build/hdhunter/apache/htdocs/* /tmp/apache/htdocs
# ipcmk -M 65536
# ../../../tools/monitor_http_param create
export __AFL_SHM=12 __AFL_SHM_SIZE=65536 __HTTP_PARAM=13 __HTTP_PARAM_SIZE=328 __EXECUTION_PATH=14 __EXECUTION_PATH_SIZE=8
export LD_LIBRARY_PATH=$(realpath ../build/hdhunter/apache/lib)
export HDHUNTER_MODE=scgi
# export HDHUNTER_TRACE=/tmp/hdhunter_trace_apache
$(realpath ../build/hdhunter/apache/bin/httpd) -X -f $(realpath ../config/debug.conf) -DFOREGROUND
# gdb --args $(realpath ../build/hdhunter/apache/bin/httpd) -X -f $(realpath ../config/debug.conf) -DFOREGROUND
