#!/bin/bash
echo "================================================="
echo "           Nyx build script"
echo "================================================="
echo

ROOT=$(pwd)

ln -s ../QEMU-Nyx ./QEMU-Nyx
ln -s ../packer ./packer

echo "[*] Checking QEMU-Nyx ..."
if [ ! -f "QEMU-Nyx/x86_64-softmmu/qemu-system-x86_64" ]; then
    cd QEMU-Nyx/ || return
    cp $ROOT/Makefile.libxdc ./libxdc/Makefile || exit 1
    ./compile_qemu_nyx.sh lto || exit 1
    cd $ROOT
fi

echo "[*] checking packer init.cpio.gz ..."
if [ ! -f "packer/linux_initramfs/init.cpio.gz" ]; then
    cd packer/linux_initramfs/ || return
    sh pack.sh || exit 1
    cd $ROOT
fi

echo "[+] All done for nyx_mode, enjoy!"

exit 0
