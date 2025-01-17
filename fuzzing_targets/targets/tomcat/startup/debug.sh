#!/bin/sh
export JAVA_HOME=$(realpath ../../../../vendors/Witcher-java/build/linux-x86_64-normal-server-release/jdk)
export PATH=$JAVA_HOME/bin:$PATH
# export CATALINA_HOME=$(realpath ../build/hdhunter/tomcat)
export CATALINA_HOME=$(realpath ../apache-tomcat-10.1.9-src/output/build)
# export CATALINA_BASE=$(realpath ../build/hdhunter/tomcat)
export CATALINA_BASE=$(realpath ../apache-tomcat-10.1.9-src/output/build)
export LD_LIBRARY_PATH=$(realpath ../build/hdhunter/lib)
# export JAVA_OPTS="-Djava.security.egd=file:/dev/./urandom -Xmx3500m" # important: fix hangs on tomcat startup
# export JDK_JAVA_OPTIONS=-XX:+WitcherInstrumentation
export HDHUNTER_WITCHER_JAVA_FILTER_PATH=$(realpath ../config/witcher_javafilters)
# ipcmk -M 65536
# ../../../tools/monitor_http_param create
export __AFL_SHM=8 __AFL_SHM_SIZE=65536 __HTTP_PARAM=9 __HTTP_PARAM_SIZE=328 __EXECUTION_PATH=10 __EXECUTION_PATH_SIZE=8
# export HDHUNTER_TRACE=/tmp/hdhunter_trace_tomcat

# Start Tomcat
$CATALINA_HOME/bin/catalina.sh run
# while ! curl http://127.0.0.1:8080; do
#     sleep 0.1
# done
