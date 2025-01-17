#!/bin/sh
export JAVA_HOME=/hdhunter/jdk
export PATH=$JAVA_HOME/bin:$PATH
export CATALINA_HOME=/hdhunter/tomcat
export CATALINA_BASE=/hdhunter/tomcat
export LD_LIBRARY_PATH=/hdhunter/lib:/hdhunter/jdk/lib
export JAVA_OPTS="-Djava.security.egd=file:/dev/./urandom" # important: fix hangs on tomcat startup
export JDK_JAVA_OPTIONS=-XX:+WitcherInstrumentation
export HDHUNTER_WITCHER_JAVA_FILTER_PATH=/hdhunter/witcher_javafilters

# Start Tomcat
$CATALINA_HOME/bin/catalina.sh start
