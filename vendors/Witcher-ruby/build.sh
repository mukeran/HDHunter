#!/bin/sh

setup_docker() {
    docker run -v $(pwd):/tmp/hdhunter-witcher-ruby-build --rm -it ubuntu:18.04 /tmp/hdhunter-witcher-ruby-build/build.sh docker
}

in_docker() {
    # Install dependencies
    sed -i 's/archive.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list
    sed -i 's/security.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list
    apt update && apt install -y build-essential autoconf zip libx11-dev libxext-dev libxrender-dev libxtst-dev libxt-dev libcups2-dev libfontconfig1-dev libasound2-dev libssl-dev libgdbm-dev libffi-dev libedit-dev libqdbm-dev bison ruby curl file

    # Build
    cd /tmp/hdhunter-witcher-ruby-build
    autoconf
    ./configure
    make
    mkdir -p _install
    make DESTDIR=$PWD/_install install
    sed -i 's/#!\/usr\/local\/bin\/ruby/#!\/usr\/bin\/env ruby/g' _install/usr/local/bin/*
}

if [ "$1" = "docker" ]; then
    in_docker
else
    setup_docker
fi

# PATH=$(realpath ./_install/usr/local/bin):$PATH RUBY_ENGINE=$(realpath ./_install/usr/local/bin/ruby) RUBYLIB=$(realpath ./_install/usr/local/lib/ruby/2.7.0):$(realpath ./_install/usr/local/lib/ruby/2.7.0/x86_64-linux) GEM_HOME=$(realpath ./_install/usr/local/lib/ruby/gems/2.7.0) GEM_PATH=$(realpath ./_install/usr/local/lib/ruby/gems/2.7.0) ruby /tmp/test.rb