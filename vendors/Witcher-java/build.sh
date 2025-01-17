#!/bin/sh

setup_docker() {
    docker run -v $(pwd):/tmp/hdhunter-witcher-java-build --rm -it ubuntu:18.04 /tmp/hdhunter-witcher-java-build/build.sh docker
}

in_docker() {
    # Install dependencies
    sed -i 's/archive.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list
    sed -i 's/security.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list
    apt update && apt install -y build-essential autoconf zip libx11-dev libxext-dev libxrender-dev libxtst-dev libxt-dev libcups2-dev libfontconfig1-dev libasound2-dev gcc-4.8 g++-4.8 curl file

    update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-4.8 50
    update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-4.8 50

    # Install JDK 10
    cd /tmp
    curl -O https://download.java.net/openjdk/jdk10/ri/openjdk-10+44_linux-x64_bin_ri.tar.gz
    tar -xvf openjdk-10+44_linux-x64_bin_ri.tar.gz
    export JAVA_HOME=/tmp/jdk-10
    export CLASSPATH=.:$JAVA_HOME/lib
    export PATH=$JAVA_HOME/bin:$PATH

    # Build
    cd /tmp/hdhunter-witcher-java-build
    bash configure --disable-warnings-as-errors
    CONF="linux-x86_64-normal-server-release" make JOBS=50
}

if [ "$1" = "docker" ]; then
    in_docker
else
    setup_docker
fi

