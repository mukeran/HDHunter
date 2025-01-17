#!/bin/sh

# This file is verfied for Ubuntu 22.04

# An existing ruby is needed
# sudo apt install build-essential autoconf ruby bison gperf libyaml-dev libffi-dev

./autogen.sh
mkdir -p build && cd build
../configure --prefix="/hdhunter/ruby"
make
mkdir -p ../_install
make DESTDIR=$(realpath ../_install) install
cd ..

# sed -i 's/\/hdhunter\/ruby\/bin\/ruby/\/usr\/bin\/env ruby/g' _install/hdhunter/ruby/bin/*
# __AFL_SHM=9 PATH=$(realpath ./_install/hdhunter/ruby/bin):$PATH RUBY_ENGINE=$(realpath ./_install/hdhunter/ruby/bin/ruby) RUBYLIB=$(realpath ./_install/hdhunter/ruby/lib/ruby/3.2.0):$(realpath ./_install/hdhunter/ruby/lib/ruby/3.2.0/x86_64-linux) GEM_HOME=$(realpath ./_install/hdhunter/ruby/lib/ruby/gems/3.2.0) GEM_PATH=$(realpath ./_install/hdhunter/ruby/lib/ruby/gems/3.2.0) ruby /tmp/test.rb