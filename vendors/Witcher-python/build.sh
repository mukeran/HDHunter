#!/bin/sh
bash ./configure --prefix=/hdhunter/python
make
mkdir -p _install
make DESTDIR=$PWD/_install install

sed -i 's/#!\/hdhunter\/python\/bin\/python3/#!\/usr\/bin\/env python3/g' _install/hdhunter/python/bin/*
# __AFL_SHM=9 PATH=$(realpath ./_install//hdhunter/python/bin):$PATH PYTHONPATH=$(realpath ./_install/hdhunter/python/lib/python3.7/site-packages) python3 --version