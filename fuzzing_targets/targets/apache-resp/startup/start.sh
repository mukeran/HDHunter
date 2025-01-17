#!/bin/sh
LD_LIBRARY_PATH=/hdhunter/apache/lib/ nohup /hdhunter/apache/bin/httpd -X > /tmp/apache.log 2>&1 &
