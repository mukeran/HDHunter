#!/bin/sh

mkdir -p build/hdhunter/tomcat/

# Copy Witcher-java environment
cp -Lr $TARGETS_ROOT/../vendors/Witcher-java/build/linux-x86_64-normal-server-release/jdk build/hdhunter/

# Copy Witcher-java dependencies
python3 $TARGETS_ROOT/tools/copy_dependencies.py build/hdhunter/jdk/bin/java build/hdhunter/jdk/lib/
python3 $TARGETS_ROOT/tools/copy_dependencies.py build/hdhunter/jdk/lib/server/libjvm.so build/hdhunter/jdk/lib/

# Generate and copy witcher_javafilters

# Old version (with every class, too much and too slow)
# python3 $TARGETS_ROOT/tools/generate_javafilters.py apache-tomcat-10.1.9-src/output/classes config/witcher_javafilters
# python3 $TARGETS_ROOT/tools/generate_javafilters.py -a $TARGETS_ROOT/../vendors/Witcher-java/build/linux-x86_64-normal-server-release/jdk/modules/java.net.http config/witcher_javafilters
# python3 $TARGETS_ROOT/tools/generate_javafilters.py -a $TARGETS_ROOT/../vendors/Witcher-java/build/linux-x86_64-normal-server-release/jdk/modules/jdk.httpserver config/witcher_javafilters
# python3 $TARGETS_ROOT/tools/generate_javafilters.py -a -f http $TARGETS_ROOT/../vendors/Witcher-java/build/linux-x86_64-normal-server-release/jdk/modules/java.security.jgss config/witcher_javafilters
# python3 $TARGETS_ROOT/tools/generate_javafilters.py -a -f http $TARGETS_ROOT/../vendors/Witcher-java/build/linux-x86_64-normal-server-release/jdk/modules/java.base config/witcher_javafilters
cp config/witcher_javafilters build/hdhunter/

# Copy Tomcat
cp -r apache-tomcat-10.1.9-src/output/build/* build/hdhunter/tomcat/
rm -rf build/hdhunter/tomcat/webapps/*
rm -rf build/hdhunter/tomcat/webapps-javaee/*

# Copy application
cp config/ROOT.war build/hdhunter/tomcat/webapps-javaee/

# Copy HDHunter Runtime
mkdir -p build/hdhunter/lib/
cp $TARGETS_ROOT/../target/debug/libhdhunter_rt_java.so build/hdhunter/lib/
python3 $TARGETS_ROOT/tools/copy_dependencies.py build/hdhunter/lib/libhdhunter_rt_java.so build/hdhunter/lib/

# Pack
tar -cvf $TARGETS_ROOT/dist/tomcat.tar build
tar -rvf $TARGETS_ROOT/dist/tomcat.tar -C startup setup.sh start.sh check_payload port timeout
gzip -f $TARGETS_ROOT/dist/tomcat.tar

# Remove Witcher-java environment
rm -rf build/hdhunter/jdk
