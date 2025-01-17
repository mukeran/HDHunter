#!/bin/sh

# Download and setup ant
if [ ! -d apache-ant-1.10.14 ]; then
    echo "Downloading apache-ant-1.10.14"
    curl -O https://downloads.apache.org/ant/binaries/apache-ant-1.10.14-bin.tar.gz
    tar -xvf apache-ant-1.10.14-bin.tar.gz
    rm apache-ant-1.10.14-bin.tar.gz
fi

export ANT_HOME=$(pwd)/apache-ant-1.10.14
export PATH=$PATH:$(pwd)/apache-ant-1.10.14/bin

# Setup Java (built modified Witcher-java)
export JAVA_HOME=$(realpath $TARGETS_ROOT/../vendors/Witcher-java/build/linux-x86_64-normal-server-release/jdk)
export CLASSPATH=.:$JAVA_HOME/lib
export PATH=$JAVA_HOME/bin:$PATH

# Copy hdhunter-api-java
mkdir -p /tmp/hdhunter-tomcat-build
cp $TARGETS_ROOT/runtime/java/hdhunter-api-java/target/hdhunter-api-java-1.0-SNAPSHOT.jar /tmp/hdhunter-tomcat-build/

cp config/build.properties ./apache-tomcat-10.1.9-src/build.properties

# Build
cd apache-tomcat-10.1.9-src
ant clean
ant
